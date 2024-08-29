use clap::Parser;
use msg800::{self, tunel};
use std::str::FromStr;
use tokio::net::{TcpListener, TcpStream};
use std::env;

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

    let mut mode = tunel::Mode::from_str(&args.mode).unwrap();
    let source_addr = format!("{}:{}", args.source_host, args.source_port);
    let target_addr = format!("{}:{}", args.target_host, args.target_port);

    if let tunel::Mode::ENCRYPT(key, iv) = &mut mode {
        key.push_str(&env::var("MSG800_TUNEL_ENC_KEY").expect("tunel need enc key"));
        iv.push_str(&env::var("MSG800_TUNEL_ENC_IV").expect("tunel need enc iv"));
    }

    if let tunel::Mode::DECRYPT(key, iv) = &mut mode {
        key.push_str(&env::var("MSG800_TUNEL_DEC_KEY").expect("tunel need dec key"));
        iv.push_str(&env::var("MSG800_TUNEL_DEC_IV").expect("tunel need dec iv"));
    }

    let listener = TcpListener::bind(&source_addr).await.unwrap();

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let target_addr = target_addr.clone();
        let mode = mode.clone();
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
