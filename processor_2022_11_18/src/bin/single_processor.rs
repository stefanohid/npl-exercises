use clap::Parser;
use pcap::{Capture};
use etherparse::*;
use etherparse::{NetSlice::*, TransportSlice::*};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

#[derive(Serialize, Deserialize, Debug)]
struct Msg {
    ip_ctr: i32,
    tcp_ctr: i32,
    tcp_options_ctr: i32,
    ip_options_ctr: i32,
}

#[derive(Parser, Debug)]
struct Cli {
    /// Network interface
    #[arg(long, short, default_value = "eth0", required = true)]
    interface: String,
    /// Server host
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
    /// Server port
    #[arg(long, short, default_value_t = 10000)]
    port: u16,
}

fn main() {
    let cli = Cli::parse();
    let mut cap = Capture::from_device(cli.interface.as_str())
        .expect("Can't set network device")
        .promisc(true)
        .snaplen(5000)
        .immediate_mode(true)
        .open()
        .expect("Can't open device for capture");

    cap.filter("ip or ip6", true).unwrap();

    let (tx, rx) = std::sync::mpsc::channel();

    let capture_thread = std::thread::spawn(move || {
        while let Ok(packet) = cap.next_packet() {
            let sliced_packet = SlicedPacket::from_ethernet(&packet.data);
            let mut ip_ctr: i32 = 0;
            let mut tcp_ctr: i32 = 0;
            let mut tcp_options_ctr: i32 = 0;
            let mut ip_options_ctr: i32 = 0;

            match sliced_packet {
                Err(value) => println!("Err {:?}", value),
                Ok(value) => {
                    
                    match value.net {
                        Some(Ipv4(ipv4)) => {
                            ip_ctr += 1;
                            if false == ipv4.header().options().is_empty() {
                                ip_options_ctr += 1;
                            }
                        }
                        Some(Ipv6(ipv6)) => {
                            ip_ctr += 1;
                             if !ipv6.extensions().is_empty() {
                                ip_options_ctr += 1;
                            }
                        }
                        Some(Arp(_value)) => {},
                        None => {}
                    }

                    match value.transport {
                        Some(Icmpv4(_value)) => {},
                        Some(Icmpv6(_value)) => {},
                        Some(Udp(_value)) => {},
                        Some(Tcp(value)) => {
                            tcp_ctr += 1;
                            let options: Vec<Result<TcpOptionElement, TcpOptionReadError>> =
                                value.options_iterator().collect();

                            if !(options.is_empty() || options.iter().all(|_option| options.is_empty())) {
                                tcp_options_ctr += 1;
                            }
                        }
                        None => {}
                    }
                }
                
            }

            let msg = Msg {ip_ctr: ip_ctr, ip_options_ctr: ip_options_ctr, tcp_ctr: tcp_ctr, tcp_options_ctr: tcp_options_ctr};
            let serialized = serde_json::to_string(&msg).unwrap();
            //println!("Sent: {}", serialized.as_str());
            tx.send(serialized).unwrap();
        }
    });

    let stats_thread = std::thread::spawn(move || {
        let mut last_print = Instant::now();
        let mut tot_ip_ctr: i32 = 0;
        let mut tot_tcp_ctr: i32 = 0;
        let mut tot_tcp_options_ctr: i32 = 0;
        let mut tot_ip_options_ctr: i32 = 0;
        loop {
            match rx.recv_timeout(Duration::from_millis(500)) {
                Ok(serialized) => {
                    let msg: Msg = serde_json::from_str(&serialized).unwrap();

                    tot_ip_ctr += msg.ip_ctr;
                    tot_tcp_ctr += msg.tcp_ctr;
                    tot_ip_options_ctr += msg.ip_options_ctr;
                    tot_tcp_options_ctr += msg.tcp_options_ctr;
                }

                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    // No message arrived, but continue to the timer check.
                }

                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
            }

            if last_print.elapsed() >= Duration::from_secs(5) {
                println!("\nSTATISTICS:");
                println!("Total IP Packets: {}", tot_ip_ctr);
                println!("Total TCP Packets: {}", tot_tcp_ctr);
                println!("Total IP Options Filled: {}", tot_ip_options_ctr);
                println!("Total TCP Options Filled: {}", tot_tcp_options_ctr);
                println!("Percentage of filled IP Options: {}%", (tot_ip_options_ctr as f64 / tot_ip_ctr as f64 * 100.0));
                println!("Percentage of filled TCP Options: {}%", (tot_tcp_options_ctr as f64 / tot_tcp_ctr as f64 * 100.0));
                last_print = std::time::Instant::now();
            }
        }
    });

    capture_thread.join().unwrap();
    stats_thread.join().unwrap();

}