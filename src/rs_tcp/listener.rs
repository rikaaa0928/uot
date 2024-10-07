use tokio::net::{TcpListener};
use std::io::{Error, ErrorKind};
use log::debug;
use crate::rs_tcp::crypt;
use crate::rs_tcp::stream::RStream;


pub struct RSListener {
    inner: TcpListener,
    key: String,
}

impl RSListener {
    pub async fn bind(addr: String, key: String) -> Result<Self, Error> {
        let listener = TcpListener::bind(addr).await?;
        Ok(Self { inner: listener, key })
    }

    pub async fn accept(&mut self) -> Result<RStream, Error> {
        let (stream, _) = self.inner.accept().await?;
        let mut res = RStream::new(stream, self.key.clone());
        // let mut buf = [0; 10];
        // let n = stream.read(&mut buf).await?;
        //
        // if n != 10 || &buf[..n] != self.key.as_bytes() {
        //     stream.shutdown().await?;
        //     drop(stream);
        //     return Err(Error::new(ErrorKind::Other, "Invalid key"));
        // }
        // let x = res.read().await;
        // if x.is_err() {
        //     res.close().await?;
        //     drop(res);
        //     return Err(x.err().unwrap());
        // }
        // let a = x.unwrap();
        // let auth_str = std::str::from_utf8(&a).unwrap();
        // debug!("auth_str: {}", auth_str);
        // let auth_info = crypt::parse_auth(auth_str);
        // if auth_info.is_err() {
        //     res.close().await?;
        //     drop(res);
        //     return Err(Error::new(ErrorKind::Other, "Invalid key, parse error"));
        // }
        // if auth_info.unwrap().pw != self.key {
        //     res.close().await?;
        //     drop(res);
        //     return Err(Error::new(ErrorKind::Other, "Invalid key"));
        // }
        Ok(res)
    }
}