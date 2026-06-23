use clap::Parser;
use serde::{Deserialize, Serialize};
use std::net::{SocketAddr, UdpSocket};

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

#[derive(Serialize, Deserialize, Debug)]
enum Message {
    Query,
    Response(String),
}

fn main() {
    let cli = Cli::parse();

    let clt_sock: SocketAddr = "0.0.0.0:0".parse().unwrap();
    let srv_sock: SocketAddr = format!("{}:{}", cli.host, cli.port).parse().unwrap();

    let sock = UdpSocket::bind(clt_sock).expect("Unable to open socket");
    verbose_log(
        cli.verbose,
        1,
        format!("UDP client started on {}", sock.local_addr().unwrap()),
    );

    let query = serde_json::to_string(&Message::Query).unwrap();
    sock.send_to(query.as_bytes(), srv_sock)
        .expect("Can't send query");

    let mut buffer = [0_u8; 1500];
    let (nbytes, sender) = sock.recv_from(&mut buffer).unwrap();
    verbose_log(
        cli.verbose,
        2,
        format!("Received {} bytes from {}", nbytes, sender),
    );

    let response: Message = serde_json::from_slice(&buffer[..nbytes]).unwrap();

    match response {
        Message::Response(ip) => println!("{ip}"),
        Message::Query => eprintln!("Unexpected query received from server"),
    }
}

fn verbose_log(verbose: u8, level: u8, message: String) {
    if verbose >= level {
        eprintln!("{message}");
    }
}