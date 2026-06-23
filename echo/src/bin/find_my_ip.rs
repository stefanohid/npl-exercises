use serde::{Serialize, Deserialize};
use std::net::{UdpSocket, SocketAddr};
use clap::Parser;


#[derive(Parser, Debug)]
struct Cli {
    /// Server host
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
    /// Server port
    #[arg(short, long, default_value_t = 10000)]
    port: u16,
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[derive(Serialize, Deserialize)]
    enum Message {
        Query,
        Response(String),
    }

fn main() {
    let cli = Cli::parse();
    let srv_sock = SocketAddr::new(cli.host.parse().unwrap(), cli.port);
    let sock = UdpSocket::bind(srv_sock).expect("Unable to open socket");
    
    loop {
        let mut buffer = [0_u8;1500];
        let (_nbytes, clt) = sock.recv_from(&mut buffer).unwrap();
        let msg = Message::Response(format!("Your IP: {}", clt.ip()));
        let json_str = serde_json::to_string(&msg).unwrap();

        let _ = sock.send_to(json_str.as_bytes(), clt).expect("Can't send response");
    }
}