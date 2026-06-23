use clap::{ArgGroup, Parser};
use std::path::PathBuf;
use pcap::{Capture};

#[derive(Parser, Debug)]
#[command(group(
    ArgGroup::new("input")
        .required(true)
        .args(["interface", "file"])
))]
struct Cli {
    /// Network interface to read from
    #[arg(short = 'i', long)]
    interface: Option<String>,

    /// File to read from
    #[arg(short = 'f', long)]
    file: Option<PathBuf>,
    
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}



fn main() {

    let cli = Cli::parse();

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

fn count_protocols<T>(cap: &mut Capture<T>) where T: pcap::Activated {
    let mut tcp_ctr = 0;
    let mut udp_ctr = 0;

    let pkt_handler = |packet: pcap::Packet| {
        let protocol = packet.data[23];
        println!("Protocol: {} \n", protocol);
        if protocol == 6 {
            tcp_ctr += 1;
        } else if protocol == 17 {
            udp_ctr += 1;
        }
    };

    cap.for_each(Some(100), pkt_handler).unwrap();
    println!("Classifica Pacchetti:\n");
    // print them in a way that the one that has more packets is on top, and the one that has less packets is on the bottom.
    if tcp_ctr > udp_ctr {
        println!("TCP packets: {}", tcp_ctr);
        println!("UDP packets: {}", udp_ctr);
    } else {
        println!("UDP packets: {}", udp_ctr);
        println!("TCP packets: {}", tcp_ctr);
    }
}
 
fn verbose_log(verbose: u8, level: u8, message: String) {
    if verbose >= level {
        eprintln!("{message}");
    }
}
