use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::sync::mpsc;
use pcap::Capture;
use etherparse;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use clap::Parser;

/// Contare quanti pacchetti e quanti byte porta ciascun flusso.
/// Un flusso è source ip, destination ip, source port, destination port, protocol.
/// Vogliamo contare i top k flow senders (partiamo dai pacchetti soltanto)
/// Possiamo usare le hashmap pre contare i pacchetti. La direzione non ci interessa, quindi possiamo
/// ordinare da min a max gli ip e usare la coppia come chiave della hashmap.

#[derive(Parser, Debug)]
struct Cli {
    /// Top k senders
    #[arg(short = 'k', default_value_t = 5)]
    k: usize,
}

fn main () {
    let cli = Cli::parse();

    let mut cap = Capture::from_device("eth0")
        .expect("Can't capture from device")
        .promisc(true)
        .immediate_mode(true)
        .open()
        .unwrap();

    let (tx, rx) = mpsc::channel();

    let capture_thread = std::thread::spawn(move || {
        while let Ok(packet) = cap.next_packet() {
            let h = etherparse::PacketHeaders::from_ethernet_slice(&packet.data).unwrap();
            match h.net {
                Some(etherparse::NetHeaders::Ipv4(h, _)) => {
                    let source = IpAddr::V4(Ipv4Addr::from(h.source));
                    let destination = IpAddr::V4(Ipv4Addr::from(h.destination));
                    let protocol = h.protocol;

                    tx.send((destination, source, protocol)).unwrap();
                }

                Some(etherparse::NetHeaders::Ipv6(h, _)) => {
                    let source = IpAddr::V6(Ipv6Addr::from(h.source));
                    let destination = IpAddr::V6(Ipv6Addr::from(h.destination));
                    let protocol = h.next_header;

                    tx.send((destination, source, protocol)).unwrap();
                }

                _ => {
                    println!("Not an IP packet");
                }
            }
        }
    });

    let counter_thread = std::thread::spawn(move || {
        let mut last_print = Instant::now();
        let mut flow_map = HashMap::new();

        loop {
            match rx.recv() {
                Ok((destination, source, protocol)) => {
                    println!("{:?} - {:?}. Protocol: {:?}\n", destination, source, protocol);

                    let min_ip = std::cmp::min(source, destination);
                    let max_ip = std::cmp::max(source, destination);

                    let k = format!("{} -> {}", min_ip, max_ip);
                    *flow_map.entry(k).or_insert(0) += 1;
                }
                Err(mpsc::RecvError) => break,
            }

            if last_print.elapsed() >= Duration::from_secs(5) {
                let mut flow_vec: Vec<(&String, &i32)> = flow_map.iter().collect();
                flow_vec.sort_by(|a, b| b.1.cmp(&a.1));
                println!("\x1b[31mTOP PACKET SENDERS:\x1b[0m");
                for i in 0..cli.k {
                    if let Some((flow, count)) = flow_vec.get(i) {
                        println!("\x1b[31m{}) {} - {} packets\x1b[0m", i + 1, flow, count);
                    }
                }
                last_print = std::time::Instant::now();
                println!("\n");
            }
        }
    });

    capture_thread.join().unwrap();
    counter_thread.join().unwrap();
}