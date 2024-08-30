use clap::Parser;
use msg800::proxy::Socks5;
use msg800::Result;
use tokio::net::{TcpListener, TcpStream};

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, default_value_t = 8082)]
    port: u16,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let addr = format!("127.0.0.1:{}", args.port);

    let listener = TcpListener::bind(&addr).await.unwrap();

    println!("socks5 listen on {addr}");

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            let _ = process(socket).await;
        });
    }
}

async fn process(socket: TcpStream) -> Result<()> {
    let mut socks5 = Socks5::new(socket);
    let _ = socks5.process().await?;
    Ok(())
}
