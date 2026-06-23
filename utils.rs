use std::collections::HashMap;
use std::time::{Duration, Instant};
use clap::{ArgGroup, Parser};

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

fn main() {

}

fn read_from_stdin() {
    // do it in a loop if you need to keep it active
    io::stdin()
        .read_line(&mut username)
        .expect("Could not read username");
}

fn verbose_log(verbose: u8, level: u8, message: String) {
    if verbose >= level {
        eprintln!("{message}");
    }
}

fn capture() {
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

            count_protocols(&mut cap);
        }
        (None, Some(file)) => {
            verbose_log(
                cli.verbose,
                1,
                format!("Reading from file: {}", file.display()),
            );
            let mut cap = Capture::from_file(file)
                .unwrap();
            count_protocols(&mut cap);
        }
        _ => unreachable!(), // This should never happen due to the ArgGroup
    }
}

fn count_protocols<T>(cap: &mut Capture<T>) where T: pcap::Activated {}

fn etherparse() {
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

        println!(
            "{}:{} -> {}:{}",
            source_ip, source_port, destination_ip, destination_port
        );
}

fn packet_handler() {
    let pkt_handler = |packet: pcap::Packet| {
        let src_ip = IpAddr::from([packet.data[26], packet.data[27], packet.data[28], packet.data[29]]);
        let dst_ip = IpAddr::from([packet.data[30], packet.data[31], packet.data[32], packet.data[33]]);
        let protocol = packet.data[23];
        println!("Packet timestamp: {} seconds and {} microseconds", packet.header.ts.tv_sec, packet.header.ts.tv_usec);
        println!("Packet snaplen: {} bytes", packet.header.caplen);
        println!("Packet Length: {} bytes", packet.header.len);

        println!("[{:?}] ---> [{:?}] - Protocol: {} \n", src_ip, dst_ip, protocol);
    };
}

fn print_timeout() {
    let mut last_print = Instant::now();
    let mut flow_map = HashMap::new();

    let k = format!("{}:{} -> {}:{}", flow.src_ip, flow.src_port, flow.dst_ip, flow.dst_port);
    *flow_map.entry(k).or_insert(0) += 1;

    loop {
        if last_print.elapsed() >= Duration::from_secs(20) {
            let mut flow_vec: Vec<(&String, &i32)> = flow_map.iter().collect();
            flow_vec.sort_by(|a, b| b.1.cmp(&a.1));
            println!("\nTOP PACKET SENDERS:");
            for i in 0..5 {
                if let Some((flow, count)) = flow_vec.get(i) {
                    println!("{}) {} - {} packets", i + 1, flow, count);
                }
            }
            last_print = std::time::Instant::now();
            flow_map.clear();
            println!("\n");
        }
    }
}