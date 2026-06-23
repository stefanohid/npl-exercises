use std::net::{SocketAddr, UdpSocket};
use serde::{Deserialize, Serialize};
use std::net::ToSocketAddrs;

use clap::{ArgGroup, Parser};

#[derive(Parser, Debug)]
#[command(group(
    ArgGroup::new("input")
        .required(true)
        .args(["interface", "pcap_file"])
))]
struct Cli {
    /// Client host
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
    /// Interface
    #[arg(long, short, default_value = "eth0")]
    interface: String,
    /// Interval time
    #[arg(long, short = 'n', default_value_t = 5)]
    interval: i32,
    /// Read from pcap file or not
    #[arg(long, short = 'p')]
    pcap_file: bool,
    /// Verbose mode
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[derive(Serialize, Deserialize, Debug)]
struct DNS {
    dname: String,
    qtype: String,
}

fn main() {
    let cli = Cli::parse();
    let srv_sock: SocketAddr = format!("{}:53", cli.host).parse().unwrap();
    let sock = UdpSocket::bind(srv_sock).expect("Could not bind");

    loop {
        let mut buffer = [0_u8;1500];
        let (nbytes, clt) = sock.recv_from(&mut buffer).unwrap();
        let received = std::str::from_utf8(&buffer[..nbytes]).unwrap();
        let json_object: DNS = serde_json::from_str(&received).unwrap();
        let domain = json_object.dname;
        
        if domain.is_empty() {
            sock.send_to("Empty domain name".as_bytes(), clt).expect("Could not send message");
        }

        println!("Received the following query type: {}", json_object.qtype);
        let domain_addr = format!("{}:0", domain);
        
        let addrs: Vec<String> = ToSocketAddrs::to_socket_addrs(&domain_addr)
        .unwrap()
        .map(|addr| addr.to_string())
        .collect();

        let response = addrs.join("\n");

        sock.send_to(response.as_bytes(), clt)
            .expect("Could not send response");
        println!("Sent response to the client");
    }
}