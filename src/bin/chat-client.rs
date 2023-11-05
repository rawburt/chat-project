use clap::Parser;
use tokio::{io::stdin, net::TcpStream};
use tokio_stream::StreamExt;
use tokio_util::codec::{Framed, FramedRead, LinesCodec};

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
    let mut server_frame = Framed::new(tcp_stream, LinesCodec::new_with_max_length(1024));

    // read stdin
    let stdin = stdin();
    let mut stdin_frame = FramedRead::new(stdin, LinesCodec::new());

    // main client loop
    loop {
        tokio::select! {
            stdin_result = stdin_frame.next() => match stdin_result {
                None => panic!("reader next None"),
                Some(Err(_)) => panic!("reader error"),
                Some(Ok(input)) => println!("STDIN: {}", input),
            },
            stream_result = server_frame.next() => match stream_result {
                None => panic!("server conn next None"),
                Some(Err(_)) => panic!("server_conn error"),
                Some(Ok(message)) => println!("STREAM: {}", message),
            }
        }
    }
}
