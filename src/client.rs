use std::{env, process};
use std::io::ErrorKind;
use std::net::{SocketAddr};
use std::sync::{Arc};
use tokio::sync::Mutex;
use std::thread::sleep;
use std::time::Duration;
use anyhow::Error;
use log::{debug, error, warn};
use tokio::net::{UdpSocket};
use tokio::spawn;
use tokio::time::timeout;
use crate::rs_tcp::stream::{RSReadHalf, RStream, RSWriteHalf};

pub async fn start(l_addr: String, d_addr: String, auth: String) -> crate::Result<()> {
    let sock = UdpSocket::bind(l_addr).await?;
    let ur = Arc::new(sock);
    let uw = ur.clone();

    let stream = RStream::connect(d_addr.clone(), auth.clone()).await?;
    let (tr, tw) = RStream::split(stream);
    // let tcp_stream: Option<RStream> = None;
    let reader: Arc<Mutex<RSReadHalf>> = Arc::new(Mutex::new(tr));
    let writer: Arc<Mutex<RSWriteHalf>> = Arc::new(Mutex::new(tw));
    let mut last_addr: Option<SocketAddr> = None;
    loop {
        let mut buf = [0; 65536];
        let (size, addr) = ur.recv_from(&mut buf).await?;
        let w_buf = &buf[..size];
        debug!("client udp recv {:?} {}",&addr,size);
        let reader = reader.clone();
        let uw = uw.clone();
        let last_addr_none = last_addr.is_none();
        if last_addr_none {
            last_addr.replace(addr);
            spawn(async move {
                loop {
                    let reader = reader.clone();
                    let uw = uw.clone();
                    let mut reader_mutex = reader.lock().await;
                    let read_f = reader_mutex.read();
                    let res =
                        timeout(Duration::from_secs(60 * 60), read_f).await;
                    if res.is_err() {
                        error!("client tcp {:?} read timeout {:?}", &addr,res.unwrap_err());
                        break;
                    }
                    let data = res.unwrap();
                    if data.is_err() {
                        error!("client tcp {:?} read error {:?}", &addr,data.unwrap_err());
                        break;
                    }
                    let res = uw.send_to(&data.unwrap(), &addr).await;
                    if res.is_err() {
                        error!("client udp {:?} send error {:?}", &addr,res.unwrap_err());
                        break;
                    }
                }
                process::exit(1);
            });
        } else if last_addr.clone().unwrap() != addr {
            error!("addr changed {:?} {:?}",&last_addr, &addr);
            return Err(Error::new(std::io::Error::new(ErrorKind::Other, "closed 2")));
        }
        {
            let res = writer.clone().lock().await.write(w_buf).await;
            if res.is_err() {
                error!("client tcp write error {:?}",res.unwrap_err());
            } else {
                debug!("client tcp write {}",res.unwrap());
            }
        }
    }

    Ok(())
}

// pub async fn start(l_addr: String, d_addr: String, auth: String) -> crate::Result<()> {
//     let sock = UdpSocket::bind(l_addr).await?;
//     let ur = Arc::new(sock);
//     let uw = ur.clone();
//     // let tcp_stream: Option<RStream> = None;
//     let mut reader: Arc<Mutex<Option<Arc<Mutex<RSReadHalf>>>>> = Arc::new(Mutex::new(None));
//     let mut writer: Option<Arc<Mutex<RSWriteHalf>>> = None;
//     let last_addr: Option<SocketAddr> = None;
//     loop {
//         let mut buf = [0; 65536];
//         let (size, addr) = ur.recv_from(&mut buf).await?;
//         let w_buf = &buf[..size];
//         let mut reader = reader.clone();
//         let mut uw = uw.clone();
//         let reader_none = {
//             reader.clone().lock().await.is_none()
//         };
//         if reader_none {
//             let stream = RStream::connect(d_addr.clone(), auth.clone()).await?;
//             let (tr, tw) = RStream::split(stream);
//             {
//                 reader.clone().lock().await.replace(Arc::new(Mutex::new(tr)));
//             }
//             {
//                 writer.replace(Arc::new(Mutex::new(tw)));
//             }
//             spawn(async move {
//                 loop {
//                     let mut reader = reader.clone();
//                     let mut uw = uw.clone();
//                     let reader_unlock_opt = reader.lock().await.clone();
//                     let reader_unlock_some = reader_unlock_opt.unwrap().clone();
//                     let mut reader_mutex = reader_unlock_some.lock().await;
//                     let read_f = reader_mutex.read();
//                     let res =
//                         timeout(Duration::from_secs(60 * 60), read_f).await;
//                     if res.is_err() {
//                         warn!("client tcp {:?} read timeout", &addr);
//                         break;
//                     }
//                     let data = res.unwrap();
//                     if data.is_err() {
//                         error!("client tcp {:?} read error", &addr);
//                         break;
//                     }
//                     let res = uw.send_to(&data.unwrap(), &addr).await;
//                     if res.is_err() {
//                         error!("client udp {:?} send error", &addr);
//                         break;
//                     }
//                 }
//             });
//         }
//         if last_addr.is_none() || last_addr.unwrap() != addr {
//             error!("addr changed {:?} {:?}",&last_addr, &addr);
//             return Err(Error::new(std::io::Error::new(ErrorKind::Other, "closed 2")));
//         }
//         {
//             let res = writer.clone().unwrap().clone().lock().await.write(w_buf).await;
//             if res.is_err() {
//                 error!("client tcp write error");
//             } else {
//                 debug!("client tcp write {}",res.unwrap());
//             }
//         }
//     }
//
//     Ok(())
// }

#[tokio::test]
async fn client_test2() -> Result<(), Error> {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "debug")
    }
    env_logger::init();
    spawn(async {
        let _ = start("127.0.0.1:8980".to_string(), "127.0.0.1:8989".to_string(), "test".to_string()).await;
    });

    sleep(Duration::from_secs(1));
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    let res = socket.connect("127.0.0.1:8980").await;
    if res.is_err() {
        println!("server udp connect {}", res.err().unwrap());
        return Ok(());
    }
    socket.send("test".as_bytes()).await?;
    let mut read_buf = vec![0; 128];
    let (len, _) = socket.recv_from(&mut read_buf).await?;
    let read_buf = &read_buf[..len];
    let read_buf = String::from_utf8(read_buf.to_vec()).unwrap();
    println!("client udp recv {:?}", read_buf);


    // let mut stream = RStream::connect("127.0.0.1:8989".to_string(), "test".to_string()).await?;
    // let res = stream.write("sdgdtjgyjsefes".as_bytes()).await;
    // println!("client tcp write {} bytes", res.unwrap());
    // let res = stream.read().await;
    // println!("client tcp read {:?}", res.unwrap());
    Ok(())
}