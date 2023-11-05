use clap::Parser;
use tokio::{
    io::{stdin, BufReader},
    net::TcpStream,
};
// use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio_stream::StreamExt;
use tokio_util::codec::{Framed, FramedRead, LinesCodec};

struct ServerConn {
    framed: Framed<TcpStream, LinesCodec>,
    // sender: UnboundedSender<String>,
    // receiver: UnboundedReceiver<String>,
}

impl ServerConn {
    pub fn new(tcp_stream: TcpStream) -> Self {
        let framed = Framed::new(tcp_stream, LinesCodec::new_with_max_length(1024));
        // let (sender, receiver) = unbounded_channel();
        Self {
            framed,
            // sender,
            // receiver,
        }
    }
}

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
    let mut server_conn = ServerConn::new(tcp_stream);

    // read stdin
    let stdin = stdin();
    let mut reader = FramedRead::new(stdin, LinesCodec::new());

    // main client loop
    loop {
        tokio::select! {
            stdin_result = reader.next() => match stdin_result {
                None => panic!("reader next None"),
                Some(Err(_)) => panic!("reader error"),
                Some(Ok(input)) => println!("STDIN: {}", input),
            },
            stream_result = server_conn.framed.next() => match stream_result {
                None => panic!("server conn next None"),
                Some(Err(_)) => panic!("server_conn error"),
                Some(Ok(message)) => println!("STREAM: {}", message),
            }
        }
    }
}
