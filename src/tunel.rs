use tokio::io::ErrorKind;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt, Error, ReadHalf, WriteHalf};
use tokio::net::TcpStream;

use crate::msg::Message;
use strum_macros::EnumString;

#[derive(EnumString, Clone, Copy)]
pub enum Mode {
    FORWARD,
    ENCRYPT,
    DECRYPT,
}

fn reverse(mode: &Mode) -> Mode {
    match mode {
        Mode::FORWARD => Mode::FORWARD,
        Mode::ENCRYPT => Mode::DECRYPT,
        Mode::DECRYPT => Mode::ENCRYPT,
    }
}

const BUF_SIZE: usize = 4096;

async fn read(
    src: &mut ReadHalf<&mut TcpStream>,
    buf: &mut [u8],
    mode: &Mode,
) -> io::Result<usize> {
    match mode {
        Mode::FORWARD => src.read(buf).await,
        Mode::ENCRYPT => src.read(buf).await,
        Mode::DECRYPT => {
            let mut msg = Message::new();
            msg.unpack(src).await?;
            let bytes = msg.as_bytes();
            if bytes.len() > buf.len() {
                Err(Error::new(ErrorKind::Other, "buf too small!"))
            } else {
                for i in 0..bytes.len() {
                    buf[i] = bytes[i];
                }

                Ok(bytes.len())
            }
        }
    }
}

async fn pipe(
    src: &mut ReadHalf<&mut TcpStream>,
    dest: &mut WriteHalf<&mut TcpStream>,
    mode: Mode,
) -> Result<(), std::io::Error> {
    let mut buf = [0; BUF_SIZE];
    loop {
        match read(src, &mut buf, &mode).await {
            Ok(len) if len > 0 => match mode {
                Mode::FORWARD => dest.write_all(&buf[0..len]).await?,
                Mode::ENCRYPT => {
                    let mut msg = Message::new();
                    msg.write_bytes(&buf[0..len]);
                    let msg = msg.pack();
                    dest.write_all(&msg).await?
                }
                Mode::DECRYPT => dest.write_all(&buf[0..len]).await?,
            },
            _ => {
                dest.shutdown().await?;
                break Ok::<(), std::io::Error>(());
            }
        }
    }
}

pub async fn bridge(src: &mut TcpStream, dest: &mut TcpStream, mode: Mode) -> crate::Result<()> {
    let (mut src_read, mut src_write) = io::split(src);
    let (mut dest_read, mut dest_write) = io::split(dest);

    let dest_to_src = async { pipe(&mut dest_read, &mut src_write, reverse(&mode)).await };
    let src_to_dest = async { pipe(&mut src_read, &mut dest_write, mode).await };

    match tokio::try_join!(src_to_dest, dest_to_src) {
        Err(e) => Err(Box::new(e)),
        _ => Ok(()),
    }
}
