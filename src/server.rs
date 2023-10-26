use anyhow::anyhow;
use chat_project::{
    parser::{parse_incoming, Command, IncomingMsg, ParsedAction},
    server_state::{ServerState, User},
};
use futures::SinkExt;
use std::{net::SocketAddr, sync::Arc};
use tokio::{
    net::TcpStream,
    sync::{
        mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
        Mutex,
    },
};
use tokio_stream::StreamExt;
use tokio_util::codec::{Framed, LinesCodec};

struct Client {
    socket_addr: SocketAddr,
    framed: Framed<TcpStream, LinesCodec>,
    sender: UnboundedSender<String>,
    receiver: UnboundedReceiver<String>,
    name: Option<String>,
}

impl Client {
    pub fn new(tcp_stream: TcpStream, socket_addr: SocketAddr) -> Self {
        let framed = Framed::new(tcp_stream, LinesCodec::new());
        let (sender, receiver) = unbounded_channel();
        Self {
            socket_addr,
            framed,
            sender,
            receiver,
            name: None,
        }
    }

    // TODO: make a send_error interface

    pub async fn send_string(&mut self, string: String) -> anyhow::Result<()> {
        self.framed.send(string).await?;
        Ok(())
    }

    pub fn set_name(&mut self, name: String) {
        self.name = Some(name);
    }
}

#[derive(Debug)]
enum ClientAction {
    Quit,
    Parsed(ParsedAction),
}

async fn client_action(framed: &mut Framed<TcpStream, LinesCodec>) -> anyhow::Result<ClientAction> {
    match framed.next().await {
        // disconnected
        None => Ok(ClientAction::Quit),
        // error reading stream
        Some(Err(e)) => Err(anyhow!(e)),
        // received data from client
        Some(Ok(input)) => Ok(ClientAction::Parsed(parse_incoming(&input))),
    }
}

async fn client_registration(
    server_state: Arc<Mutex<ServerState>>,
    client: &mut Client,
) -> anyhow::Result<bool> {
    // wait for a NAME in order to register the client
    loop {
        match client_action(&mut client.framed).await? {
            ClientAction::Quit => return Ok(false),
            ClientAction::Parsed(parsed_action) => {
                match parsed_action {
                    // received NAME <user-name>
                    ParsedAction::Process(IncomingMsg::Name(name)) => {
                        let mut state = server_state.lock().await;
                        match state.add_user(name.clone(), User::new(client.sender.clone())) {
                            Ok(()) => {
                                client.set_name(name);
                                return Ok(true);
                            }
                            // error trying to save client with given user name. possible error is duplicate user name.
                            Err(server_error) => {
                                client.send_string(server_error.to_string()).await?
                            }
                        }
                    }
                    // received NAME with errors
                    ParsedAction::Error(Command::Name, parse_error) => {
                        client.send_string(parse_error.to_string()).await?
                    }
                    // received QUIT
                    ParsedAction::Process(IncomingMsg::Quit) => return Ok(false),
                    // ignore commands other than NAME and QUIT
                    ParsedAction::Process(_) | ParsedAction::Error(_, _) | ParsedAction::None => {}
                }
            }
        }
    }
}

async fn client_teardown(
    server_state: Arc<Mutex<ServerState>>,
    client: &Client,
) -> anyhow::Result<()> {
    if let Some(name) = &client.name {
        let mut state = server_state.lock().await;
        if let Err(e) = state.remove_user(name) {
            return Err(anyhow!(e));
        }
    }
    Ok(())
}

// TODO: logging on each incoming command
/// The entry point for a new client connection to the server.
pub async fn client_connection(
    server_state: Arc<Mutex<ServerState>>,
    tcp_stream: TcpStream,
    socket_addr: SocketAddr,
) -> anyhow::Result<()> {
    // create new client
    let mut client = Client::new(tcp_stream, socket_addr);

    // wait for a NAME in order to register the client and user into the server state
    let registered = client_registration(server_state.clone(), &mut client).await?;

    // if the client wasn't registered then they quit or a connection was lost
    if !registered {
        return Ok(());
    }

    // main client loop
    loop {
        tokio::select! {
            // handle outgoing data to client
            Some(message) = client.receiver.recv() => {
                client.send_string(message).await?;
            }
            result = client_action(&mut client.framed) => match result {
                // some kind of bad thing happened. remove the client from the state and raise an error.
                Err(e) => {
                    client_teardown(server_state.clone(), &client).await?;
                    return Err(anyhow!(e))
                },
                // exit the loop for proper state cleanup
                Ok(ClientAction::Quit) => break,
                Ok(ClientAction::Parsed(parsed_action)) => match parsed_action {
                    // QUIT - exit the loop for proper state cleanup
                    ParsedAction::Process(IncomingMsg::Quit) => break,
                    // NAME <user-name> - rename the client
                    ParsedAction::Process(IncomingMsg::Name(name)) => {
                        let mut state = server_state.lock().await;
                        match state.rename_user(&client.name.clone().unwrap(), &name) {
                            Ok(()) => {
                                // change client name if server successfully changes state
                                client.set_name(name);
                            },
                            Err(server_error) => {
                                client.send_string(server_error.to_string()).await?;
                            }
                        }
                    },
                    // JOIN <room-name> - join a room
                    ParsedAction::Process(IncomingMsg::Join(room)) => {
                        let mut state = server_state.lock().await;
                        match state.join_room(room, client.name.clone().unwrap()) {
                            Ok(()) => {},
                            Err(server_error) => {
                                client.send_string(server_error.to_string()).await?;
                            }
                        }
                    },
                    // SAY <room-name> <message> - send a message to a room
                    ParsedAction::Process(IncomingMsg::SayRoom(room, message)) => {
                        let mut state = server_state.lock().await;
                        match state.say_to_room(&client.name.clone().unwrap(), &room, message) {
                            Ok(()) => {},
                            Err(server_error) => {
                                client.send_string(server_error.to_string()).await?;
                            }
                        }
                    }
                    // SAY <user-name> <message> - send a message to another client
                    ParsedAction::Process(IncomingMsg::SayUser(user, message)) => {
                        let state = server_state.lock().await;
                        match state.say_to_user(&client.name.clone().unwrap(), &user, message) {
                            Ok(()) => {},
                            Err(server_error) => {
                                client.send_string(server_error.to_string()).await?;
                            }
                        }
                    },
                    // ROOMS - list all rooms
                    ParsedAction::Process(IncomingMsg::Rooms) => {
                        let state = server_state.lock().await;
                        for room in state.rooms() {
                            client.send_string(format!("ROOM {}", room)).await?;
                        }
                    },
                    // LEAVE <room-name> - leave a room
                    ParsedAction::Process(IncomingMsg::Leave(_room)) => {
                        todo!();
                    },
                    // USERS <room-name> - list all users in a room
                    ParsedAction::Process(IncomingMsg::Users(room)) => {
                        let state = server_state.lock().await;
                        match state.users(&room) {
                            Ok(users) => {
                                for user in users {
                                    client.send_string(format!("USER {}", user)).await?;
                                }
                            }
                            Err(server_error) => {
                                client.send_string(server_error.to_string()).await?;
                            }
                        }
                    },
                    // send any command parsing errors to the client
                    ParsedAction::Error(_, parse_error) => {
                        client.send_string(parse_error.to_string()).await?
                    }
                    // empty and unknown commands are ignored
                    ParsedAction::None => {}
                }
            }
        }
    }

    // remove the client from the server state
    client_teardown(server_state.clone(), &client).await?;

    Ok(())
}
