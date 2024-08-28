use clap::Parser;
use msg800::{self, tunel};
use std::str::FromStr;
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

    let mode = tunel::Mode::from_str(&args.mode).unwrap();
    let source_addr = format!("{}:{}", args.source_host, args.source_port);
    let target_addr = format!("{}:{}", args.target_host, args.target_port);

    println!("local listen on: {source_addr}");
    println!("forward to: {target_addr}");

    let listener = TcpListener::bind(&source_addr).await.unwrap();

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let target_addr = target_addr.clone();
        tokio::spawn(async move {
            let _ = process(socket, &target_addr, mode).await;
        });
    }
}

async fn process(mut src: TcpStream, target_addr: &str, mode: tunel::Mode) -> msg800::Result<()> {
    let mut dest = TcpStream::connect(target_addr).await?;
    msg800::tunel::bridge(&mut src, &mut dest, mode).await?;
    Ok(())
}
