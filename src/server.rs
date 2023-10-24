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

#[derive(Debug)]
enum ClientAction {
    Quit,
    Parsed(ParsedAction),
}

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

    pub async fn next(&mut self) -> anyhow::Result<ClientAction> {
        match self.framed.next().await {
            // disconnected
            None => Ok(ClientAction::Quit),
            // error reading stream
            Some(Err(e)) => Err(anyhow!(e)),
            // received data from client
            Some(Ok(input)) => Ok(ClientAction::Parsed(parse_incoming(&input))),
        }
    }

    pub async fn send_string(&mut self, string: String) -> anyhow::Result<()> {
        self.framed.send(string).await?;
        Ok(())
    }

    pub fn set_name(&mut self, name: String) {
        self.name = Some(name);
    }
}

async fn client_registration(
    server_state: Arc<Mutex<ServerState>>,
    client: &mut Client,
) -> anyhow::Result<bool> {
    // wait for a NAME in order to register the client and user into the server state
    loop {
        match client.next().await? {
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

/// The entry point for a new client connection to the server.
pub async fn client_connection(
    server_state: Arc<Mutex<ServerState>>,
    tcp_stream: TcpStream,
    socket_addr: SocketAddr,
) -> anyhow::Result<()> {
    // create new client
    let mut client = Client::new(tcp_stream, socket_addr);

    // wait for a NAME in order to register the client and user into the server state
    let registered = client_registration(server_state, &mut client).await?;

    // if the client wasn't registered then they quit or a connection was lost
    if !registered {
        return Ok(());
    }

    // main client loop

    // handle incoming data from client
    // handle outgoing data to client

    Ok(())
}
