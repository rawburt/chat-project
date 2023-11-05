use clap::Parser;
use tokio::{io::stdin, net::TcpStream};
use tokio_stream::StreamExt;
use tokio_util::codec::{Framed, FramedRead, LinesCodec};
use futures::SinkExt;

async fn client(tcp_stream: TcpStream) -> anyhow::Result<()> {
    // read/write to socket stream
    let mut server_frame = Framed::new(tcp_stream, LinesCodec::new_with_max_length(1024));

    // read stdin
    let stdin = stdin();
    let mut stdin_frame = FramedRead::new(stdin, LinesCodec::new());

    // main client loop
    loop {
        println!("loop");
        tokio::select! {
            stdin_result = stdin_frame.next() => match stdin_result {
                None => {
                    println!("STDIN stream closed.");
                    return Ok(());
                },
                Some(Err(e)) => {
                    println!("Unknown error with STDIN stream. {}", e);
                    return Ok(());
                },
                Some(Ok(input)) => server_frame.send(input).await?,
            },
            stream_result = server_frame.next() => match stream_result {
                None => {
                    println!("Server disconnected.");
                    return Ok(());
                },
                Some(Err(e)) => {
                    println!("Unknown error with server stream. {}", e);
                    return Ok(());
                },
                Some(Ok(message)) => println!("{}", message),
            }
        }
    }
}

#[derive(Parser)]
#[command(author, version, long_about = None)]
struct Cli {
    address: String,
}

// TODO: spawn two threads: 
//  - one thread for server frame io. 
//  - one frame for blocking stdin io.
// ref: https://docs.rs/tokio/latest/tokio/io/struct.Stdin.html
// 
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // parse command line arguments
    let cli = Cli::parse();

    // connect to server
    let tcp_stream = TcpStream::connect(cli.address).await?;

    // run client app
    client(tcp_stream).await?;

    
    println!("Exiting.");

    Ok(())
}
