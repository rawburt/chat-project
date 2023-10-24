use chat_project::server_state::ServerState;
use clap::Parser;
use std::{io, sync::Arc};
use tokio::{net::TcpListener, sync::Mutex};

pub mod server;

#[derive(Parser)]
#[command(author, version, long_about = None)]
struct Cli {
    address: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // parse command line arguments
    let cli = Cli::parse();

    // initialize logging
    env_logger::init();

    // the shared server state amongst all connected clients
    let server_state = Arc::new(Mutex::new(ServerState::new()));

    // socket bind to address
    let listener = TcpListener::bind(&cli.address).await?;

    log::info!("listening for connections on {}", cli.address);

    loop {
        // accept new client connection
        let (stream, addr) = listener.accept().await?;
        // clone references to shared server state
        let server_state = server_state.clone();

        // spawn new async process
        tokio::spawn(async move {
            log::info!("client connection accepted {}", addr);
            if let Err(e) = server::client_connection(server_state, stream, addr).await {
                log::info!("error = {:?}", e);
            }
            log::info!("client connection closed {}", addr);
        });
    }
}
