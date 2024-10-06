mod client;
mod rs_tcp;
mod server;

use std::env;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use anyhow::Error;
use clap::{arg, command, Arg, ArgAction};

type Result<T, E = anyhow::Error> = std::result::Result<T, E>;

#[tokio::main]
async fn main() -> Result<()> {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info")
    }
    env_logger::init();
    let matches = command!()
        .arg(Arg::new("server")
                 .short('s')
                 .long("server")
                 .action(ArgAction::SetTrue)
                 .help("server mod"), )
        .arg(arg!([src] "addr src. client mod [udp-ip:udp-port]; server mod [tcp-ip:tcp-port]"))
        .arg(arg!([dst] "addr dst. client mod [tcp-ip:tcp-port]; server mod [udp-port]"))
        .arg(arg!([sec] "secret"))
        // .arg(arg!([bind] "port bind. client mod [remote-udp-port]; server mod none"))
        .get_matches();
    let src_opt = matches.get_one::<String>("src");
    let dst_opt = matches.get_one::<String>("dst");
    let auth = matches.get_one::<String>("sec");
    let s_mod = matches.get_one::<bool>("server");
    if s_mod.is_some() && *s_mod.unwrap() {
        let _ = server::start(src_opt.unwrap().to_string(), dst_opt.unwrap().to_string(), auth.unwrap().to_string()).await?;
    } else {
        let _ = client::start(src_opt.unwrap().to_string(), dst_opt.unwrap().to_string(), auth.unwrap().to_string()).await?;
    }
    Ok(())
}

#[tokio::test]
async fn stop_test() -> std::result::Result<(), Error> {
    let stop = Arc::new(AtomicBool::new(false));
    let stop2 = stop.clone();
    println!("{}",stop.load(std::sync::atomic::Ordering::Relaxed));
    println!("{}",stop2.load(std::sync::atomic::Ordering::Relaxed));
    stop2.store(true, std::sync::atomic::Ordering::Relaxed);
    println!("{}",stop.load(std::sync::atomic::Ordering::Relaxed));
    println!("{}",stop2.load(std::sync::atomic::Ordering::Relaxed));

    Ok(())
}