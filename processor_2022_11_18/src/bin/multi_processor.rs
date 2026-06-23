use clap::Parser;
use etherparse::*;
use etherparse::{NetSlice::*, TransportSlice::*};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use pnet::datalink::{self, NetworkInterface, Config, Channel::Ethernet, FanoutOption, FanoutType};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
struct Msg {
    worker_id: usize,
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
    let interface = get_interface(cli.interface.as_str());
    let mut t = vec![];
    let (tx, rx) = std::sync::mpsc::channel();

    for i in 1..6 {
        let iface = interface.clone();
        let thread_tx = tx.clone();
        let mut config = Config::default();
        let fanout: FanoutOption = FanoutOption {
            group_id: 1234,
            fanout_type: FanoutType::HASH, 
            defrag: true, 
            rollover: false, 
        }; 

        config.linux_fanout = Some(fanout);

        t.push(std::thread::spawn(move || {
            let mut rx = match datalink::channel(&iface, config) {
                Ok(Ethernet(_, rx)) => rx,
                Ok(_) => panic!("unhandled channel type"),
                Err(e) => panic!("channel error: {e}"),
            };

            loop {
                match rx.next() {
                    Ok(packet) => {

                        let sliced_packet = SlicedPacket::from_ethernet(packet);
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

                        let msg = Msg {worker_id: i, ip_ctr: ip_ctr, ip_options_ctr: ip_options_ctr, tcp_ctr: tcp_ctr, tcp_options_ctr: tcp_options_ctr};
                        let serialized = serde_json::to_string(&msg).unwrap();
                        //println!("Sent: {}", serialized.as_str());
                        thread_tx.send(serialized).unwrap();

                    },
                    Err(e) => eprintln!("Error: {:?}", e), 
                } 
            } 
        }));
    }  

    let stats_thread = std::thread::spawn(move || {
        let mut last_print = Instant::now();
        let mut map: HashMap<usize, Msg> = (1..=5)
            .map(|id| {
                (
                    id,
                    Msg {
                        worker_id: id,
                        ip_ctr: 0,
                        tcp_ctr: 0,
                        tcp_options_ctr: 0,
                        ip_options_ctr: 0,
                    },
                )
            })
            .collect();
        let mut tot_ip_ctr: i32 = 0;
        let mut tot_tcp_ctr: i32 = 0;
        let mut tot_tcp_options_ctr: i32 = 0;
        let mut tot_ip_options_ctr: i32 = 0;

        loop {
            match rx.recv_timeout(Duration::from_millis(500)) {
                Ok(serialized) => {
                    let msg: Msg = serde_json::from_str(&serialized).unwrap();

                    let stats = map.get_mut(&msg.worker_id).unwrap();
                    stats.ip_ctr += msg.ip_ctr;
                    stats.tcp_ctr += msg.tcp_ctr;
                    stats.ip_options_ctr += msg.ip_options_ctr;
                    stats.tcp_options_ctr += msg.tcp_options_ctr;
                }

                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    // No message arrived, but continue to the timer check.
                }

                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
            }

            if last_print.elapsed() >= Duration::from_secs(5) {
                println!("\nWORKER STATISTICS:");

                for i in 1..6 {
                    let wrk_ip_ctr: i32 = map.get(&i).unwrap().ip_ctr;
                    let wrk_tcp_ctr: i32 = map.get(&i).unwrap().tcp_ctr;
                    let wrk_tcp_options_ctr: i32 = map.get(&i).unwrap().tcp_options_ctr;
                    let wrk_ip_options_ctr: i32 = map.get(&i).unwrap().ip_options_ctr;

                    tot_ip_ctr += wrk_ip_ctr;
                    tot_tcp_ctr += wrk_tcp_ctr;
                    tot_ip_options_ctr += wrk_ip_options_ctr;
                    tot_tcp_options_ctr += wrk_tcp_options_ctr;

                    println!("\nWorker number {i}:");
                    println!("IP Packets: {}", wrk_ip_ctr);
                    println!("TCP Packets: {}", wrk_tcp_ctr);
                    println!("IP Options Filled: {}", wrk_ip_options_ctr);
                    println!("TCP Options Filled: {}", wrk_tcp_options_ctr);
                    println!("Percentage of filled IP Options: {:.2}%",
                        percentage(wrk_ip_options_ctr, wrk_ip_ctr)
                    );
                    println!(
                        "Percentage of filled TCP Options: {:.2}%",
                        percentage(wrk_tcp_options_ctr, wrk_tcp_ctr)
                    );
                }

                println!("\nAGGREGATE STATISTICS:");
                println!("Total IP Packets: {}", tot_ip_ctr);
                println!("Total TCP Packets: {}", tot_tcp_ctr);
                println!("Total IP Options Filled: {}", tot_ip_options_ctr);
                println!("Total TCP Options Filled: {}", tot_tcp_options_ctr);
                println!("Percentage of filled IP Options: {:.2}%",
                    percentage(tot_ip_options_ctr, tot_ip_ctr)
                );
                println!(
                    "Percentage of filled TCP Options: {:.2}%\n\n",
                    percentage(tot_tcp_options_ctr, tot_tcp_ctr)
                );
                last_print = std::time::Instant::now();
            }
        }
    });


    for i in t {
        let _ = i.join();
    }

    stats_thread.join().unwrap();

}

fn get_interface(name: &str) -> NetworkInterface {
    datalink::interfaces()
    .into_iter()
    .find(|iface| iface.name == name)
    .expect("interface not found")
}

fn percentage(part: i32, total: i32) -> f64 {
    if total == 0 {
        0.0
    } else {
        part as f64 / total as f64 * 100.0
    }
}