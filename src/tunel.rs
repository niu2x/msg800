use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

const BUF_SIZE: usize = 4096;

pub async fn bridge(mary: &mut TcpStream, bob: &mut TcpStream) -> crate::Result<()> {
    let (mut mary_read, mut mary_write) = io::split(mary);
    let (mut bob_read, mut bob_write) = io::split(bob);

    let req = async {
        let mut buf = [0; BUF_SIZE];

        let ret = loop {
            match bob_read.read(&mut buf).await {
                Ok(len) if len > 0 => mary_write.write_all(&buf[0..len]).await?,
                _ => {
                    mary_write.shutdown().await?;
                    break Ok::<(), std::io::Error>(());
                }
            }
        };

        println!("req finished");
        ret
    };

    let resp = async {
        let mut buf = [0; BUF_SIZE];
        let ret = loop {
            match mary_read.read(&mut buf).await {
                Ok(len) if len > 0 => bob_write.write_all(&buf[0..len]).await?,
                _ => {
                    bob_write.shutdown().await?;
                    break Ok::<(), std::io::Error>(());
                }
            }
        };
        println!("resp finished");
        ret
    };

    match tokio::try_join!(req, resp) {
        Err(e) => Err(Box::new(e)),
        _ => Ok(()),
    }
}
