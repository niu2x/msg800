use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use strum_macros::EnumString;

#[derive(EnumString, Clone, Copy)]
pub enum Mode {
    FORWARD,
}

const BUF_SIZE: usize = 4096;

pub async fn bridge(src: &mut TcpStream, dest: &mut TcpStream, mode: Mode) -> crate::Result<()> {
    let (mut src_read, mut src_write) = io::split(src);
    let (mut dest_read, mut dest_write) = io::split(dest);

    let src_to_dest = async {
        let mut buf = [0; BUF_SIZE];

        loop {
            match dest_read.read(&mut buf).await {
                Ok(len) if len > 0 => match mode {
                    Mode::FORWARD => src_write.write_all(&buf[0..len]).await?,
                },
                _ => {
                    src_write.shutdown().await?;
                    break Ok::<(), std::io::Error>(());
                }
            }
        }
    };

    let dest_to_src = async {
        let mut buf = [0; BUF_SIZE];
        loop {
            match src_read.read(&mut buf).await {
                Ok(len) if len > 0 => match mode {
                    Mode::FORWARD => dest_write.write_all(&buf[0..len]).await?,
                },
                _ => {
                    dest_write.shutdown().await?;
                    break Ok::<(), std::io::Error>(());
                }
            }
        }
    };

    match tokio::try_join!(src_to_dest, dest_to_src) {
        Err(e) => Err(Box::new(e)),
        _ => Ok(()),
    }
}
