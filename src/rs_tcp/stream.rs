use tokio::net::{TcpStream};
use std::io::{Error, ErrorKind};
use std::time::{Duration};
use log::debug;
use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, split, WriteHalf};
use tokio::time::{sleep, timeout};
use crate::rs_tcp::crypt;

pub struct RStream {
    pub stream: TcpStream,
    key: String,
    // pub time: Instant,
}

pub struct RSReadHalf {
    stream: ReadHalf<TcpStream>,
}

/// The writable half of a value returned from [`split`](split()).
pub struct RSWriteHalf {
    stream: WriteHalf<TcpStream>,
}

impl RSReadHalf {
    pub async fn read(&mut self) -> Result<Vec<u8>, Error> {
        let mut buf = [0; 8];
        let n = self.stream.read(&mut buf).await?;
        if n != 8 {
            return Err(Error::new(ErrorKind::Other, "closed 1"));
        }
        let len = u64::from_be_bytes(buf);
        // let mut data = Vec::with_capacity(len as usize);
        let mut data = vec![0; len as usize];
        let n = timeout(Duration::from_secs(1), self.stream.read_exact(&mut data)).await??;
        // let n = self.stream.read_exact(&mut data).await?;
        if n == 0 {
            return Err(Error::new(ErrorKind::Other, "closed 2"));
        }
        if n != len as usize {
            return Err(Error::new(ErrorKind::Other, "read data len error"));
        }
        let decrypted_data = crypt::decrypt(&data, len as u8);
        // self.time = Instant::now();
        Ok(decrypted_data)
    }
}

impl RSWriteHalf {
    pub async fn write(&mut self, data: &[u8]) -> Result<usize, Error> {
        let len = data.len();
        self.stream.write_all(&u64::to_be_bytes(len as u64)).await?;
        // sleep(Duration::from_secs(2)).await;
        let encrypted_data = crypt::encrypt(data, len as u8);
        self.stream.write_all(&encrypted_data).await?;
        // self.time = Instant::now();
        Ok(len)
    }

    pub async fn close(&mut self) -> Result<(), Error> {
        debug!("RStream close");
        self.stream.shutdown().await?;
        Ok(())
    }
}

impl RStream {
    pub(crate) fn new(stream: TcpStream, key: String) -> RStream {
        RStream { stream, key }
    }
    pub async fn connect(addr: String, key: String) -> io::Result<RStream> {
        let stream = TcpStream::connect(addr).await?;
        // let key_vec: &[u8] = key.as_bytes();
        let mut res = RStream { stream, key: key.clone() };
        let auth_info = crypt::generate_auth(key.clone());
        // stream.write_all(key_vec).await?;
        res.write(auth_info.as_bytes()).await?;
        Ok(res)
    }

    pub async fn write(&mut self, data: &[u8]) -> Result<usize, Error> {
        let len = data.len();
        self.stream.write_all(&u64::to_be_bytes(len as u64)).await?;
        // sleep(Duration::from_secs(2)).await;
        let encrypted_data = crypt::encrypt(data, len as u8);
        self.stream.write_all(&encrypted_data).await?;
        // self.time = Instant::now();
        Ok(len)
    }

    pub async fn read(&mut self) -> Result<Vec<u8>, Error> {
        let mut buf = [0; 8];
        let n = self.stream.read(&mut buf).await?;
        if n != 8 {
            return Err(Error::new(ErrorKind::Other, "closed 1"));
        }
        let len = u64::from_be_bytes(buf);
        // let mut data = Vec::with_capacity(len as usize);
        let mut data = vec![0; len as usize];
        let n = timeout(Duration::from_secs(1), self.stream.read_exact(&mut data)).await??;
        // let n = self.stream.read_exact(&mut data).await?;
        if n == 0 {
            return Err(Error::new(ErrorKind::Other, "closed 2"));
        }
        if n != len as usize {
            return Err(Error::new(ErrorKind::Other, "read data len error"));
        }
        let decrypted_data = crypt::decrypt(&data, len as u8);
        // self.time = Instant::now();
        Ok(decrypted_data)
    }
    pub async fn close(&mut self) -> Result<(), Error> {
        debug!("RStream close");
        self.stream.shutdown().await?;
        Ok(())
    }
    pub fn split(stream: RStream) -> (RSReadHalf, RSWriteHalf)
    {
        let (r, w) = split(stream.stream);
        return (RSReadHalf { stream: r }, RSWriteHalf { stream: w });
    }
}

#[tokio::test]
async fn test_test() -> Result<(), anyhow::Error> {
    let len = 3;
    let be_len = u64::to_be_bytes(len as u64);
    println!("{:?}", be_len);
    let r_len = u64::from_be_bytes(be_len);
    println!("{:?}", r_len);
    println!("{:?}", 255 as u8);
    let x = 256;
    println!("{:?}", x as u8);
    let x = 257;
    println!("{:?}", x as u8);
    Ok(())
}