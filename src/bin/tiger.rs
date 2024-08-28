use clap::Parser;
use msg800::v5::{self, Socks5};
use std::error::Error;
use tokio::net::{TcpListener, TcpStream};

#[derive(Parser, Debug)]
struct Args {
    #[arg(short)]
    port: u16,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let listener = TcpListener::bind(format!("127.0.0.1:{}", args.port))
        .await
        .unwrap();

    loop {
        let (socket, _) = listener.accept().await.unwrap();

        tokio::spawn(async move {
            let _ = process(socket).await;
        });
    }
}

async fn process(socket: TcpStream) -> Result<(), Box<dyn Error>> {
    let mut socks5 = Socks5::new(socket, v5::Mode::TIGER);
    let _ = socks5.process().await?;

    Ok(())
}
