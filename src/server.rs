use std::env;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::thread::sleep;
use anyhow::Error;
use log::{debug, error, warn};
use tokio::net::UdpSocket;
use tokio::spawn;
use tokio::time::timeout;
// use tokio::time::error::Error;
use crate::rs_tcp::listener::RSListener;
use crate::rs_tcp::stream::RStream;

pub async fn start(l_addr: String, d_addr: String, auth: String) -> crate::Result<()> {
    let mut l = RSListener::bind(l_addr, auth).await?;

    loop {
        let s = l.accept().await;
        if s.is_err() {
            error!("l accept {}", s.err().unwrap());
            continue;
        }
        let mut s = s.unwrap();
        let d_addr = d_addr.clone();

        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        let res = socket.connect(&d_addr).await;
        if res.is_err() {
            error!("server udp connect {}", res.err().unwrap());
            let _ = s.close().await;
            break;
        }
        let r = Arc::new(socket);
        let w = r.clone();

        let (mut sr, mut sw) = RStream::split(s);

        spawn(async move {
            let stop = Arc::new(AtomicBool::new(false));
            let stop2 = stop.clone();
            spawn(async move {
                loop {
                    if stop2.load(std::sync::atomic::Ordering::Relaxed) {
                        let _ = sw.close().await;
                        break;
                    }
                    let res = timeout(std::time::Duration::from_secs(60 * 60), async {
                        let mut buf = [0; 65536];
                        let res = r.recv_from(&mut buf).await;
                        if res.is_err() {
                            error!("server udp recv {}", res.err().unwrap());
                            return;
                        }
                        let (size, _) = res.unwrap();
                        let w_buf = &buf[..size];
                        let _ = sw.write(w_buf).await;
                    }).await;
                    if res.is_err() {
                        warn!("server udp recv timeout, close");
                        stop2.store(true, std::sync::atomic::Ordering::Relaxed);
                        let _ = sw.close().await;
                        break;
                    }
                }
            });
            loop {
                // let mut s = s.unwrap();
                if stop.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }
                let data = sr.read().await;
                if data.is_err() {
                    error!("server tcp read {}", data.err().unwrap());
                    stop.store(true, std::sync::atomic::Ordering::Relaxed);
                    break;
                }
                let data = data.unwrap();
                debug!("server tcp read {:?}", data.len());

                let res = w.send(&data[..]).await;
                if res.is_err() {
                    error!("server udp send {}", res.err().unwrap());
                    // sr.close().await;
                    break;
                }
            }
            Ok::<(), Error>(())
        });
    }
    Ok(())
}

#[tokio::test]
async fn server_test() -> Result<(), Error> {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "debug")
    }
    env_logger::init();
    spawn(async {
        let res = UdpSocket::bind("127.0.0.1:8990").await.unwrap();
        let mut buf = [0; 65536];
        loop {
            let (n, addr) = res.recv_from(&mut buf).await.unwrap();
            println!("client udp recv {:?} {}", n, addr);
            let x = res.send_to("pong".as_bytes(), addr).await.unwrap();
            println!("client udp send {:?}", x);
        }
    });
    sleep(std::time::Duration::from_secs(1));
    let res = start("127.0.0.1:8989".to_string(), "127.0.0.1:8990".to_string(), "test".to_string()).await;
    println!("server start {}", res.err().unwrap());
    Ok(())
}