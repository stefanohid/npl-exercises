use clap::{ArgGroup, Parser};
use etherparse;
use pcap::Capture;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket};
use serde::{Deserialize, Serialize};

/// tcptraceroute google.com

#[derive(Parser, Debug)]
#[command(group(
    ArgGroup::new("input")
        .required(true)
        .args(["interface", "file"])
))]
struct Cli {
    /// Server host
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
    /// Server port
    #[arg(long, short, default_value_t = 12000)]
    port: u16,
    /// Interface
    #[arg(long, short, default_value = "eth0")]
    interface: Option<String>,
    /// File
    #[arg(long, short)]
    file: Option<String>,
    #[arg(long, short, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[derive(Serialize, Deserialize, Debug)]
struct Flow {
    segment_length: usize,
    src_ip: IpAddr,
    dst_ip: IpAddr,
    src_port: u16,
    dst_port: u16,
}

fn main() {
    let cli = Cli::parse();
    let port = cli.port;
    let host = cli.host;
    match (cli.interface, cli.file) {
        (Some(interface), None) => {
            verbose_log(
                cli.verbose,
                1,
                format!("Reading from network interface: {}", interface),
            );

            let mut cap = Capture::from_device(interface.as_str())
                .expect("Cannot create capture from device")
                .promisc(true)
                .snaplen(5000)
                .immediate_mode(true)
                .open()
                .unwrap();
            cap.filter("tcp", true).unwrap();

            extract_headers(&mut cap, host, port);
        }
        (None, Some(file)) => {
            verbose_log(
                cli.verbose,
                1,
                format!("Reading from file: {}", file),
            );
            let mut cap = Capture::from_file(file)
                .unwrap();
            cap.filter("tcp", true).unwrap();
            extract_headers(&mut cap, host, port);
        }
        _ => unreachable!(), // This should never happen due to the ArgGroup
    }
}

fn extract_headers<T>(cap: &mut Capture<T>, host: String, port: u16) where T: pcap::Activated {
    while let Ok(packet) = cap.next_packet() {
        let headers = match etherparse::PacketHeaders::from_ethernet_slice(packet.data) {
            Ok(headers) => headers,
            Err(error) => {
                eprintln!("Could not parse packet: {error}");
                continue;
            }
        };

        let (source_ip, destination_ip) = match headers.net {
            Some(etherparse::NetHeaders::Ipv4(ip, _)) => (
                IpAddr::V4(Ipv4Addr::from(ip.source)),
                IpAddr::V4(Ipv4Addr::from(ip.destination)),
            ),
            Some(etherparse::NetHeaders::Ipv6(ip, _)) => (
                IpAddr::V6(Ipv6Addr::from(ip.source)),
                IpAddr::V6(Ipv6Addr::from(ip.destination)),
            ),
            _ => continue,
        };

        let (source_port, destination_port) = match headers.transport {
            Some(etherparse::TransportHeader::Tcp(tcp)) => {
                (tcp.source_port, tcp.destination_port)
            }
            _ => continue,
        };

        let segment_length = match &headers.payload {
            etherparse::PayloadSlice::Tcp(payload) => payload.len(),
            _ => continue,
        };

        let flow = Flow {src_ip: source_ip, dst_ip: destination_ip, src_port: source_port, dst_port: destination_port, segment_length: segment_length};
        let serialized = serde_json::to_string(&flow).unwrap();
        send_info(serialized, &host, &port);

    }
}

fn send_info(serialized: String, host: &String, port: &u16) {
    let clt_addr: SocketAddr = "0.0.0.0:0".parse().unwrap();
    let srv_addr: SocketAddr = format!("{}:{}", host, port).parse().unwrap();
    let clt_sock = UdpSocket::bind(clt_addr).expect("Could not bind!");

    clt_sock.send_to(serialized.as_bytes(), srv_addr).expect("Could not send message");
    println!("Sent message: {}", serialized.as_str());
}

fn verbose_log(verbose: u8, level: u8, message: String) {
    if verbose >= level {
        eprintln!("{message}");
    }
}
