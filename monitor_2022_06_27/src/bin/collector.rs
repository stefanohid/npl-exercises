use clap::Parser;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Parser, Debug)]
struct Cli {
    /// Server host
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
    /// Server port
    #[arg(long, short, default_value_t = 12000)]
    port: u16,
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
    let srv_sock: SocketAddr = format!("{}:{}", cli.host, cli.port).parse().unwrap();
    let sock = UdpSocket::bind(srv_sock).expect("Could not bind!");

    sock.set_read_timeout(Some(Duration::from_secs(1)))
        .expect("Could not set timeout");

    let mut last_print = Instant::now();
    let mut flow_map = HashMap::new();

    loop {
        let mut buffer = [0_u8;1500];
        match sock.recv_from(&mut buffer) {
            Ok((nbytes, _clt)) => {
                let received = std::str::from_utf8(&buffer[..nbytes]).unwrap();
                let flow: Flow = serde_json::from_str(received).unwrap();

                let key = format!(
                    "{}:{} -> {}:{}",
                    flow.src_ip, flow.src_port, flow.dst_ip, flow.dst_port
                );

                *flow_map.entry(key).or_insert(0usize) += flow.segment_length;
            }

            Err(error)
                if error.kind() == std::io::ErrorKind::WouldBlock
                    || error.kind() == std::io::ErrorKind::TimedOut => {}

            Err(error) => panic!("Receive error: {error}"),
        }

        if last_print.elapsed() >= Duration::from_secs(20) {
            let mut flow_vec: Vec<(&String, &usize)> = flow_map.iter().collect();
            flow_vec.sort_by(|a, b| b.1.cmp(&a.1));
            println!("\nTOP HITTERS BY PACKET VOLUME:");
            for i in 0..5 {
                if let Some((flow, count)) = flow_vec.get(i) {
                    println!("{}) {} - {} bytes", i + 1, flow, count);
                }
            }
            last_print = std::time::Instant::now();
            flow_map.clear();
            println!("\n");
        }

    }
}
