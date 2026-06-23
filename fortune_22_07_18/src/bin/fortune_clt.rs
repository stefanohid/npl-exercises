use clap::Parser;
use serde::{Serialize, Deserialize};
use std::{net::{SocketAddr, UdpSocket}};

#[derive(Parser, Debug)]
struct Cli {
    /// Server host
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
    /// Server port
    #[arg(long, default_value_t = 10000)]
    port: u16,
    /// Verbose mode
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[derive(Serialize, Deserialize, Debug)]
enum Fortune {
    MessageType(String),
    Text(String),
}

fn main() {
    let cli = Cli::parse();
    let clt_addr: SocketAddr = "0.0.0.0:0".parse().unwrap();
    let srv_sock: SocketAddr = format!("{}:{}", cli.host, cli.port).parse().unwrap();

    let sock = UdpSocket::bind(clt_addr).unwrap();
    verbose_log(cli.verbose, 1, format!("Connected to UPD server on {}:{}", sock.local_addr().unwrap().ip(), cli.port));

    let response: Fortune = Fortune::MessageType(format!("REQ"));
    let json = serde_json::to_string(&response).unwrap();

    let _ = sock.send_to(json.as_bytes(), srv_sock).expect("Can't send response");
    verbose_log(cli.verbose, 1, format!("Sent request to server"));

    let mut buffer = [0_u8;1500];
    let (nbytes, _clt) = sock.recv_from(&mut buffer).unwrap();
    let received = std::str::from_utf8(&buffer[..nbytes]).unwrap();
    let msg: Fortune = serde_json::from_str(received).unwrap();

    match msg {
        Fortune::Text(text) => {
            println!("Here's your fortune: \"{}\"", text);
        }

        Fortune::MessageType(_message_type) => {
            panic!("Did not expect text message type from server");
        }
    }
}

fn verbose_log(verbose: u8, level: u8, message: String) {
    if verbose >= level {
        eprintln!("{message}");
    } 
}