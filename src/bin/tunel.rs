use clap::Parser;
use msg800::tunel::{self, Mode};
use msg800::{self, Result};
use std::env;
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

    let mode = Mode::from_str(&args.mode).unwrap();
    let source_addr = format!("{}:{}", args.source_host, args.source_port);
    let target_addr = format!("{}:{}", args.target_host, args.target_port);

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

fn get_secret(name: &str) -> [u8; 16] {
    env::var(name)
        .expect(&format!("tunel need {name}"))
        .as_bytes()
        .try_into()
        .expect(&format!("{name} should be 16 bytes"))
}

fn get_secret_key(mode: &Mode) -> ([u8; 16], [u8; 16]) {
    match mode {
        Mode::ENCRYPT => {
            let key = get_secret("MSG800_TUNEL_ENC_KEY");
            let iv = get_secret("MSG800_TUNEL_ENC_IV");
            (key, iv)
        }
        Mode::DECRYPT => {
            let key = get_secret("MSG800_TUNEL_DEC_KEY");
            let iv = get_secret("MSG800_TUNEL_DEC_IV");
            (key, iv)
        }
        Mode::FORWARD => ([0; 16], [0; 16]),
    }
}

async fn process(mut src: TcpStream, target_addr: &str, mode: Mode) -> Result<()> {
    let (key, iv) = get_secret_key(&mode);
    let mut dest = TcpStream::connect(target_addr).await?;
    let mut tunel = tunel::Tunel::new(key, iv);
    tunel.bridge(&mut src, &mut dest, mode).await?;
    Ok(())
}
