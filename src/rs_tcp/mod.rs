pub mod listener;
pub mod stream;
mod crypt;

use tokio::spawn;
use tokio::time::sleep;
use listener::RSListener;
use stream::*;

#[tokio::test]
async fn test_test() -> Result<(), anyhow::Error> {
    spawn(async {
        let l = RSListener::bind("127.0.0.1:8080".to_string(), "1234".to_string()).await;
        if l.is_err() {
            println!("l read {}", l.err().unwrap());
            return;
        }
        let mut l = l.unwrap();
        loop {
            let s = l.accept().await;
            if s.is_err() {
                println!("l accept read {}", s.err().unwrap());
                return;
            }
            spawn(async move {
                let mut s = s.unwrap();
                let data = s.read().await;
                if data.is_err() {
                    println!("l s read {}", data.err().unwrap());
                    return;
                }
                let data = data.unwrap();
                println!("l read {:?}", String::from_utf8(data));
                // s.stream.shutdown().await.expect("");


                let w = s.write("test2".as_bytes()).await;
                println!("l s write {}", w.unwrap());
                s.close().await.expect("");
            });
        }
    });
    sleep(std::time::Duration::from_secs(1)).await;
    let mut s = RStream::connect("127.0.0.1:8080".to_string(), "1234".to_string()).await?;
    // sleep(std::time::Duration::from_secs(1)).await;
    let res = s.write("test".as_bytes()).await;
    if res.is_err() {
        println!("c s write {:?}", res.err().unwrap());
        return Ok(());
    }
    println!("c write {:?}", res.unwrap());
    let data = s.read().await;
    if data.is_err() {
        println!("c s read {:?}", data.err().unwrap());
        return Ok(());
    }
    println!("c read {:?}", String::from_utf8(data.unwrap()));
    sleep(std::time::Duration::from_secs(1)).await;
    Ok(())
}