mod client;
mod rs_tcp;
mod server;

use std::env;
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
        // .arg(arg!([bind] "port bind. client mod [remote-udp-port]; server mod none"))
        .get_matches();
    let src_opt = matches.get_one::<String>("src");
    let dst_opt = matches.get_one::<String>("dst");
    let s_mod = matches.get_one::<bool>("server").unwrap();
    if *s_mod {

    }else{

    }

    Ok(())
}