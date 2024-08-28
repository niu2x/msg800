use crate::msg::Message;
use std::error::Error;
pub use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
// use tokio::try_join;
// type AuthMethods = Vec<u8>;

#[derive(Debug)]
struct AuthHeader {
    // pub version: u8,
    // pub method_num: u8,
    // pub methods: AuthMethods,
}

#[derive(Debug)]
enum Addr {
    DOMAIN(String),
}

#[derive(Debug, Clone)]
pub enum Mode {
    WOLF(String),
    TIGER,
}

#[derive(Debug)]
struct TargetAddress {
    pub unused_0: u8,
    // pub command: u8,
    pub unused_1: u8,
    pub addr_type: u8,
    pub addr: Addr,
    pub port: u16,
}

pub struct Socks5 {
    down_stream: TcpStream,
    mode: Mode,
}

type Result<T> = std::result::Result<T, Box<dyn Error>>;

impl Socks5 {
    pub fn new(socket: TcpStream, mode: Mode) -> Socks5 {
        return Socks5 {
            down_stream: socket,
            mode,
        };
    }

    pub async fn process(&mut self) -> Result<()> {
        let _ = self.read_auth().await?;

        self.resp_auth().await?;

        let target_addr = self.read_target_address().await?;

        let _ = self.resp_client_cmd(&target_addr).await?;

        let mut up_stream = self.connect_up_stream(&target_addr).await?;

        Socks5::pipe(&mut up_stream, &mut self.down_stream, &target_addr).await?;

        Ok(())
    }

    async fn pipe(
        up_stream: &mut TcpStream,
        down_stream: &mut TcpStream,
        target_addr: &TargetAddress,
    ) -> Result<()> {
        let (mut up_stream_read, mut up_stream_write) = io::split(up_stream);
        let (mut down_stream_read, mut down_stream_write) = io::split(down_stream);

        let req = async {
            let mut buf = [0; 1024];

            loop {
                match down_stream_read.read(&mut buf).await {
                    Ok(len) if len > 0 => up_stream_write.write_all(&buf[0..len]).await?,
                    _ => {
                        up_stream_write.shutdown().await?;
                        break Ok::<(), std::io::Error>(());
                    }
                }
            }
        };

        let resp = async {
            let mut buf = [0; 1024];
            loop {
                match up_stream_read.read(&mut buf).await {
                    Ok(len) if len > 0 => down_stream_write.write_all(&buf[0..len]).await?,
                    _ => {
                        down_stream_write.shutdown().await?;
                        break Ok::<(), std::io::Error>(());
                    }
                }
            }
        };

        println!("try_join {target_addr:?}");
        let _ = tokio::try_join!(req, resp);
        println!("try_join finished {target_addr:?}");
        Ok(())
    }

    async fn read_auth(&mut self) -> Result<AuthHeader> {
        let _version = self.down_stream.read_u8().await?;
        let method_num = self.down_stream.read_u8().await?;
        let _methods = if method_num > 0 {
            let mut temp = vec![0; method_num as usize];
            self.down_stream.read_exact(&mut temp).await?;
            temp
        } else {
            vec![]
        };

        Ok(AuthHeader {
            // version,
            // method_num,
            // methods,
        })
    }

    async fn resp_auth(&mut self) -> Result<()> {
        self.down_stream.write_all(b"\x05\x00").await?;
        Ok(())
    }

    async fn read_target_address(&mut self) -> Result<TargetAddress> {
        let unused_0 = self.down_stream.read_u8().await?;
        let command = self.down_stream.read_u8().await?;
        let unused_1 = self.down_stream.read_u8().await?;
        let addr_type = self.down_stream.read_u8().await?;

        if command != 1 {
            return Err("unsupport command".into());
        }

        if addr_type != 3 {
            return Err("unsupport addr_type".into());
        }

        let domain_len = self.down_stream.read_u8().await?;

        let mut domain = vec![0; domain_len as usize];
        self.down_stream.read_exact(&mut domain).await?;
        let domain = String::from_utf8(domain)?;

        let addr = Addr::DOMAIN(domain);

        let port = self.down_stream.read_u16().await?;

        Ok(TargetAddress {
            unused_0,
            // command,
            unused_1,
            addr_type,
            addr,
            port,
        })
    }

    async fn resp_client_cmd(&mut self, target_addr: &TargetAddress) -> Result<()> {
        let buf = [
            target_addr.unused_0,
            0,
            target_addr.unused_1,
            target_addr.addr_type,
        ];

        let mut msg = Message::new();
        msg.write_bytes(&buf);

        let Addr::DOMAIN(domain) = &target_addr.addr;

        msg.write_u8(domain.len() as u8);
        msg.write_bytes(domain.as_bytes());
        msg.write_u16(target_addr.port);

        self.down_stream.write_all(msg.as_bytes()).await?;

        Ok(())
    }

    async fn connect_up_stream(&mut self, target_addr: &TargetAddress) -> Result<TcpStream> {
        let Addr::DOMAIN(domain) = &target_addr.addr;
        let port = target_addr.port;

        let stream = TcpStream::connect(format!("{domain}:{port}")).await?;
        Ok(stream)
    }
}
