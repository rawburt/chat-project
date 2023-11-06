use anyhow::anyhow;
use clap::Parser;
use futures::SinkExt;
use tokio::{net::TcpStream, sync::mpsc::unbounded_channel};
use tokio_stream::StreamExt;
use tokio_util::codec::{Framed, LinesCodec};

#[derive(Parser)]
#[command(author, version, long_about = None)]
struct Cli {
    address: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // parse command line arguments
    let cli = Cli::parse();

    // connect to server
    let tcp_stream = TcpStream::connect(cli.address).await?;

    // server frame
    let mut server_frame = Framed::new(tcp_stream, LinesCodec::new_with_max_length(1024));

    // io bridge
    let (iosend, mut iorecv) = unbounded_channel();
    std::thread::spawn(move || {
        for line in std::io::stdin().lines() {
            iosend.send(line).unwrap();
        }
    });

    loop {
        tokio::select! {
            server_result = server_frame.next() => match server_result {
                None => {
                    println!("Server disconnected.");
                    return Ok(());
                },
                Some(Err(e)) => {
                    println!("Stream error: {}", e);
                    return Err(anyhow!(e));
                },
                Some(Ok(message)) => {
                    println!("{}", message);
                }
            },
            io_result = iorecv.recv() => match io_result {
                None => {
                    println!("Client disconnected.");
                    return Ok(());
                },
                Some(Err(e)) => {
                    println!("IO error: {}", e);
                    return Err(anyhow!(e));
                },
                Some(Ok(input)) => {
                    server_frame.send(input).await?;
                }
            }
        }
    }
}
