use clap::Parser;
use pcap::Capture;
use std::sync::mpsc;
use std::time::{Duration, Instant};

#[derive(Parser, Debug)]
struct Cli {
    /// Network interface to read from
    #[arg(short = 'i', long, required = true)]
    interface: String,
    
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}



fn main() {
    let cli = Cli::parse();

    verbose_log(
        cli.verbose,
        1,
        format!("Reading from network interface: {}", cli.interface),
    );

    let mut cap = Capture::from_device(cli.interface.as_str())
        .expect("Cannot create capture from device")
        .promisc(true)
        .snaplen(5000)
        .immediate_mode(true)
        .open()
        .unwrap();

    let (tx, rx) = mpsc::channel();

    let capture_thread = std::thread::spawn(move || {
        let pkt_handler = move |packet: pcap::Packet| {
            if packet.data.len() <= 23 {
                return;
            }

            let protocol = packet.data[23];
            println!("Protocol: {}\n", protocol);
            tx.send(protocol).unwrap();
        };

        cap.for_each(None, pkt_handler).unwrap();
    });

    let counter_thread = std::thread::spawn(move || {
        let mut tcp = 0;
        let mut udp = 0;
        let mut last_print = Instant::now();

        loop {
            match rx.recv_timeout(Duration::from_secs(1)) {
                Ok(protocol) => match protocol {
                    6 => tcp += 1,
                    17 => udp += 1,
                    _ => {}
                },
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }

            if last_print.elapsed() >= Duration::from_secs(5) {
                println!("TCP: {tcp}, UDP: {udp}");
                last_print = std::time::Instant::now();
            }
        }
    });

    capture_thread.join().unwrap();
    counter_thread.join().unwrap();
}

fn verbose_log(verbose: u8, level: u8, message: String) {
    if verbose >= level {
        eprintln!("{message}");
    }
}
