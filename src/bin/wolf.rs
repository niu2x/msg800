use clap::Parser;
use msg800::v5::{self, Socks5};
use std::error::Error;
use tokio::net::{TcpListener, TcpStream};

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, default_value_t = 8082)]
    port: u16,

    tiger_host: String,
    tiger_port: u16,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let addr = format!("127.0.0.1:{}", args.port);
    let tiger_addr = format!("{}:{}", args.tiger_host, args.tiger_port);

    let listener = TcpListener::bind(&addr).await.unwrap();

    println!("wolf listen on {addr}");
    println!("tiger is at: {tiger_addr}");

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let addr = tiger_addr.clone();
        tokio::spawn(async move {
            let _ = process(socket, addr).await;
        });
    }
}

async fn process(socket: TcpStream, tiger_addr: String) -> Result<(), Box<dyn Error>> {
    let mut socks5 = Socks5::new(socket, v5::Mode::WOLF(tiger_addr));
    let _ = socks5.process().await?;

    Ok(())
}
