use clap::Parser;
use msg800;
use tokio::net::{TcpListener, TcpStream};

#[derive(Parser, Debug)]
struct Args {
    source_host: String,
    source_port: u16,
    target_host: String,
    target_port: u16,
    mode: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let mode = args.mode;
    let source_addr = format!("{}:{}", args.source_host, args.source_port);
    let target_addr = format!("{}:{}", args.target_host, args.target_port);

    println!("local listen on: {source_addr}");
    println!("forward to: {target_addr}");

    let listener = TcpListener::bind(&source_addr).await.unwrap();

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let target_addr = target_addr.clone();
        tokio::spawn(async move {
            let _ = process(socket, &target_addr).await;
        });
    }
}

async fn process(mut src: TcpStream, target_addr: &str) -> msg800::Result<()> {
    let mut dest = TcpStream::connect(target_addr).await?;
    msg800::tunel::bridge(&mut src, &mut dest).await?;
    Ok(())
}
