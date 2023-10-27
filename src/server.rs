use anyhow::anyhow;
use chat_project::{
    messages::{IncomingMsg, Message, OutgoingMsg},
    parser::{parse_incoming, Command, ParsedAction},
    server_state::{ServerState, User},
};
use futures::SinkExt;
use log::info;
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

struct ClientConn {
    socket_addr: SocketAddr,
    framed: Framed<TcpStream, LinesCodec>,
    sender: UnboundedSender<OutgoingMsg>,
    receiver: UnboundedReceiver<OutgoingMsg>,
    name: Option<String>,
}

impl ClientConn {
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

    pub async fn send_message<T: Message>(&mut self, message: T) -> anyhow::Result<()> {
        info!("{} send_message --> {}", self.socket_addr, message);
        self.framed.send(message.to_string()).await?;
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
    client: &mut ClientConn,
) -> anyhow::Result<bool> {
    // wait for a NAME in order to register the client
    loop {
        match client_action(&mut client.framed).await? {
            ClientAction::Quit => return Ok(false),
            ClientAction::Parsed(parsed_action) => {
                info!(
                    "{} client_registration --> {}",
                    client.socket_addr, parsed_action
                );
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
                            Err(server_error) => client.send_message(server_error).await?,
                        }
                    }
                    // received NAME with errors
                    ParsedAction::Error(Command::Name, parse_error) => {
                        client.send_message(parse_error).await?
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
    client: &ClientConn,
) -> anyhow::Result<()> {
    if let Some(name) = &client.name {
        let mut state = server_state.lock().await;
        if let Err(e) = state.remove_user(name) {
            return Err(anyhow!(e));
        }
    }
    Ok(())
}

/// The entry point for a new client connection to the server.
pub async fn client_connection(
    server_state: Arc<Mutex<ServerState>>,
    tcp_stream: TcpStream,
    socket_addr: SocketAddr,
) -> anyhow::Result<()> {
    // create new client
    let mut client = ClientConn::new(tcp_stream, socket_addr);

    // tell the client they are connected to the server
    client.send_message(OutgoingMsg::Connected).await?;

    // wait for a NAME in order to register the client and user into the server state
    let registered = client_registration(server_state.clone(), &mut client).await?;

    // if the client wasn't registered then they quit or a connection was lost
    if !registered {
        return Ok(());
    }

    // tell the client they are registered to the server
    client.send_message(OutgoingMsg::Registered).await?;

    // main client loop
    loop {
        tokio::select! {
            // handle outgoing data to client
            Some(message) = client.receiver.recv() => {
                client.send_message(message).await?;
            }
            result = client_action(&mut client.framed) => match result {
                // some kind of bad thing happened. remove the client from the state and raise an error.
                Err(e) => {
                    client_teardown(server_state.clone(), &client).await?;
                    return Err(anyhow!(e))
                },
                // exit the loop for proper state cleanup
                Ok(ClientAction::Quit) => break,
                Ok(ClientAction::Parsed(parsed_action)) => {
                    info!("{} client_connection --> {}", client.socket_addr, parsed_action);
                    match parsed_action {
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
                                    client.send_message(server_error).await?
                                }
                            }
                        },
                        // JOIN <room-name> - join a room
                        ParsedAction::Process(IncomingMsg::Join(room)) => {
                            let mut state = server_state.lock().await;
                            match state.join_room(room, client.name.clone().unwrap()) {
                                Ok(()) => {},
                                Err(server_error) => {
                                    client.send_message(server_error).await?
                                }
                            }
                        },
                        // SAY <room-name> <message> - send a message to a room
                        ParsedAction::Process(IncomingMsg::SayRoom(room, message)) => {
                            let mut state = server_state.lock().await;
                            match state.say_to_room(&client.name.clone().unwrap(), &room, message) {
                                Ok(()) => {},
                                Err(server_error) => {
                                    client.send_message(server_error).await?
                                }
                            }
                        }
                        // SAY <user-name> <message> - send a message to another client
                        ParsedAction::Process(IncomingMsg::SayUser(user, message)) => {
                            let state = server_state.lock().await;
                            match state.say_to_user(&client.name.clone().unwrap(), &user, message) {
                                Ok(()) => {},
                                Err(server_error) => {
                                    client.send_message(server_error).await?
                                }
                            }
                        },
                        // ROOMS - list all rooms
                        ParsedAction::Process(IncomingMsg::Rooms) => {
                            let state = server_state.lock().await;
                            for room in state.rooms() {
                                client.send_message(OutgoingMsg::Room(room)).await?;
                            }
                        },
                        // LEAVE <room-name> - leave a room
                        ParsedAction::Process(IncomingMsg::Leave(room)) => {
                            let mut state = server_state.lock().await;
                            match state.leave_room(&room, &client.name.clone().unwrap()) {
                                Ok(()) => {},
                                Err(server_error) => {
                                    client.send_message(server_error).await?
                                }
                            }
                        },
                        // USERS <room-name> - list all users in a room
                        ParsedAction::Process(IncomingMsg::Users(room)) => {
                            let state = server_state.lock().await;
                            match state.users(&room) {
                                Ok(users) => {
                                    for user in users {
                                        client.send_message(OutgoingMsg::User(user)).await?;
                                    }
                                }
                                Err(server_error) => {
                                    client.send_message(server_error).await?
                                }
                            }
                        },
                        // send any command parsing errors to the client
                        ParsedAction::Error(_, parse_error) => {
                            client.send_message(parse_error).await?
                        }
                        // empty and unknown commands are ignored
                        ParsedAction::None => {}
                    }
                }
            }
        }
    }

    // remove the client from the server state
    client_teardown(server_state.clone(), &client).await?;

    Ok(())
}
