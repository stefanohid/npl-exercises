use pnet::{datalink, packet::Packet};
use clap::{Parser};

/// We use pnet to capture packets at the Data Link Layer (Layer 2) of the OSI model, which allows us to 
///  access the full packet data, including the Ethernet header. This is useful for analyzing network 
///  traffic in more detail and exploiting OS features that may not be accessible through higher-level 
///  libraries like Pcap. With pnet, we can capture packets directly from the network interface and process 
///  them in real-time, giving us more control over the packet capture and analysis process.
/// 
/// Pcap doesn't allow multi-thread capture and it's not very efficient for high-speed networks.
/// 
/// Fanout is a feature that allows multiple processes or threads to capture packets from the same network 
///  interface simultaneously. It works by distributing incoming packets across multiple capture instances.
/// You may have different Fanout groups, and each group can have multiple capture instances.
/// You manage all of this with the FanoutOption struct, which inckudes the FanoutType (e.g., Hash, RoundRobin, 
///  etc.) and the group ID. This allows you to efficiently capture.
/// defrag = true makes sure that the packets are reassembled before being processed.
/// 
/// HASH is the most common type of Fanout, which distributes packets based on a hash of the packet's header 
///  fields (e.g., source/destination IP and port). This ensures that packets from the same flow are processed 
///  by the same capture instance, which is important for maintaining the integrity of the captured data.
/// An alternative is RoundRobin (Load Balancing), but it does not preserve affinity.
#[derive(Parser, Debug)]
struct Cli {
    /// Network interface to read from
    #[arg(short = 'i', long, required = true)]
    interface: String,
    
    /// Number of packets to capture (default: capture indefinitely)
    #[arg(short = 'n', long, default_value = None)]
    num_of_packets: i32,

    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

fn main() {
    let cli = Cli::parse();
    let interface = get_interface(&cli);
    let mut config = datalink::Config::default();

    // Here we are applying the Fanout configuration to the datalink channel
    config.linux_fanout = Some(datalink::FanoutOption {
        fanout_type: datalink::FanoutType::HASH,
        group_id: 1,
        defrag: true,
        rollover: false,
    });

    let mut rx = match datalink::channel(&interface, config) {
        Ok(datalink::Channel::Ethernet(_tx, rx)) => rx,
        Ok(_) => panic!("Unhandled channel type"),
        Err(e) => panic!("Failed to create datalink channel: {}", e),
    };

    // At the next packet it's not guaranteed that the address will be the same. We can't modify the OS' ring behavior.
    // We don't have the ownership of the packet, but we can copy it. Otherwise with borrowing we have no guarantees.
    // The issue is at thee next packet, it goes for both pcap and pnet.
    // The alternative is to do processing in place.
    // Pnet has an embedded parser, otherwise we can use etherparse or directly with pcap.
    let mut ctr = 0;
    loop {
        match rx.next() {
            Ok(frame) => {
                let ethframe = pnet::packet::ethernet::EthernetPacket::new(frame).unwrap();
                //println!("Payload: {:?}", ethframe.payload());
                println!("Captured packet: {} -> {} (type: {})", ethframe.get_source(), ethframe.get_destination(), ethframe.get_ethertype());
                
                match ethframe.get_ethertype() {
                    pnet::packet::ethernet::EtherTypes::Ipv4 => {
                        if let Some(_ipv4_packet) = pnet::packet::ipv4::Ipv4Packet::new(ethframe.payload()) {
                            println!("Detected IPv4 packet!");
                        }
                    },
                    pnet::packet::ethernet::EtherTypes::Ipv6 => {
                        if let Some(_ipv6_packet) = pnet::packet::ipv6::Ipv6Packet::new(ethframe.payload()) {
                            println!("Detected IPv6 packet!");
                        }
                    },
                    _ => println!("Other Ethernet type: {:?}", ethframe.get_ethertype()),
                }
            }
            Err(e) => eprintln!("Receive error {e}"),
        }
        ctr += 1;
        if ctr >= cli.num_of_packets {
            break;
        }
    }

}

fn get_interface(cli: &Cli) -> pnet::datalink::NetworkInterface {
    datalink::interfaces()
        .into_iter()
        .find(|iface| iface.name == cli.interface)
        //.find(|iface| iface.is_up() && !iface.is_loopback() && iface.ips.len() > 0)
        .expect("No suitable network interface found")
}