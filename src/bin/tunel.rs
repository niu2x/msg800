use clap::Parser;
use msg800::{self, tunel};
use msg800::tunel::Mode;
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

    let  mode = Mode::from_str(&args.mode).unwrap();
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

fn get_secret(name: &str) -> [u8; 16]{
    env::var(name).expect(&format!("tunel need {name}"))
        .as_bytes().try_into().expect(&format!("{name} should be 16 bytes"))
}

async fn process(mut src: TcpStream, target_addr: &str, mode: Mode) -> msg800::Result<()> {
    let mut dest = TcpStream::connect(target_addr).await?;
    let key;
    let iv;

    match mode {
        Mode::ENCRYPT=> {
            key = get_secret("MSG800_TUNEL_ENC_KEY");
            iv = get_secret("MSG800_TUNEL_ENC_IV");
        },
        Mode::DECRYPT=> {
         key = get_secret("MSG800_TUNEL_DEC_KEY");
            iv = get_secret("MSG800_TUNEL_DEC_IV");
        }
        Mode::FORWARD => {
            key =[0; 16];
            iv = [0; 16];
        }
    };

    let mut tunel = tunel::Tunel::new(key, iv);
    tunel.bridge(&mut src, &mut dest, mode).await?;
    Ok(())
}
