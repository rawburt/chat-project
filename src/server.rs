use anyhow::anyhow;
use chat_project::{
    parser::{parse_incoming, IncomingMsg, ParsedAction},
    server_state::{ServerError, ServerState, User},
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

    pub async fn send_error(&mut self, error: ServerError) -> anyhow::Result<()> {
        self.framed.send(error.to_string()).await?;
        Ok(())
    }

    pub fn set_name(&mut self, name: String) {
        self.name = Some(name);
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
    'register: loop {
        match client.next().await? {
            ClientAction::Quit => return Ok(()),
            ClientAction::Parsed(parsed_action) => {
                match parsed_action {
                    // received NAME <user-name>
                    ParsedAction::Process(IncomingMsg::Name(name)) => {
                        let mut state = server_state.lock().await;
                        match state.add_user(name.clone(), User::new(client.sender.clone())) {
                            Ok(()) => {
                                client.set_name(name);
                                break 'register;
                            }
                            Err(e) => client.send_error(e).await?,
                        }
                    }
                    // received QUIT
                    ParsedAction::Process(IncomingMsg::Quit) => return Ok(()),
                    // ignore commands other than NAME and QUIT
                    ParsedAction::Process(_) | ParsedAction::Error(_, _) | ParsedAction::None => {}
                }
            }
        }
    }

    // main client loop
    Ok(())
}
