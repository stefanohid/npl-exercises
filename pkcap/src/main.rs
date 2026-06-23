use pcap::{Capture, Device};
use std::net::IpAddr;

/// Captured packets can be saved in .pacp files. Tcpdump is born with pcap support, and it is the most widely
///  used tool for capturing and analyzing network traffic.
/// Portability: The pcap format is designed to be portable across different platforms and architectures.
/// Pcap hides the complexity of low-level packet capture and provides a simple interface.
/// Pcap doesn't allow for multi-thread capture and it's not very efficient for high-speed networks.
/// There are other Rust tools such as rscap.
/// 
/// BPF (Berkeley Packet Filter) is a powerful filtering mechanism that allows you to specify which packets to
///  capture based on various criteria, such as source/destination IP addresses, ports, protocols, etc. Pcap
///  supports BPF filtering, which can help reduce the amount of captured data and improve performance.
/// 
/// We are picking traffic directly at level 2: Network Layer. Pacp hides whatever is in the Data Link Layer.
/// If we want to go more in-depth and exploit the OS features, we don't use Pcap and we need to take up packets
///  from the Data Link Layer. AF_PACKET is a Linux-specific socket type that allows you to capture packets at 
///  the Data Link Layer (Layer 2) of the OSI model.
/// The library we can use is called pnet.
fn main() {
    let device = Device::lookup()
        .expect("Failed to lookup default device")
        .expect("No default device found");

    let mut cap = Capture::from_device(device) // you can specify the device name here, e.g., "eth0" or "en0"
        .expect("Cannot create capture from device")
        .promisc(true) // Enable promiscuous mode to capture all packets, not just those addressed to the host.
        .snaplen(5000) // Set the snapshot length to 5000 bytes (max. length for packet)
        //.timeout(1000)
        .immediate_mode(true) // Enable immediate mode to capture packets as soon as they arrive, without buffering.
        .open()
        .unwrap();

    // let mut cap = Capture::from_file("/home/stefa/npl-projects/pkcap/traces/smallFlows.pcap")
    //     .unwrap();

    println!("Link type: {:?}", cap.get_datalink()); // Print the data link type of the capture (e.g., Ethernet, Wi-Fi, etc.). =1 is usually Ethernet.
    cap.filter("ip", true).unwrap(); // Set a BPF filter to capture only IP packets.

    // let mut cnt = 0;
    // while let Ok(packet) = cap.next_packet() {
    //     println!("Got a packet! {:?}", &packet.data[12..14]); // Print the EtherType field (bytes 12 and 13), which indicates the protocol of the packet (e.g., 0x0800 for IPv4, 0x86DD for IPv6, etc.)
    //     // Why do we use & in &packet? Because packet is a reference to a Packet struct, and we want to
    //     //  access its data field, which is a byte slice. The & operator is used to dereference the packet
    //     //  reference and access its data field. If we didn't use &, we would be trying to access the data
    //     //  field of the reference itself, which would not work. 
    //     println!("{:?} ---> {:?} - Protocol: {} \n", &packet.data[26..30], &packet.data[30..34], &packet.data[23]); 
    //         // Print the source IP address (bytes 26-29), destination IP address (bytes 30-33), and protocol (byte 23) of the packet.
    //     cnt += 1;
    //     if cnt >= 100 {
    //         break; // Stop after capturing 100 packets
    //     }
    // }

    let pkt_handler = |packet: pcap::Packet| {
        // let src_ip = &packet.data[26..30];
        // let dst_ip = &packet.data[30..34];
        // let protocol = packet.data[23];

        let src_ip = IpAddr::from([packet.data[26], packet.data[27], packet.data[28], packet.data[29]]);
        let dst_ip = IpAddr::from([packet.data[30], packet.data[31], packet.data[32], packet.data[33]]);
        let protocol = packet.data[23];
        println!("Packet timestamp: {} seconds and {} microseconds", packet.header.ts.tv_sec, packet.header.ts.tv_usec);
        println!("Packet snaplen: {} bytes", packet.header.caplen);
        println!("Packet Length: {} bytes", packet.header.len);
        // What's the difference between length and snaplen? The length field represents the actual length
        //  of the packet on the network, while the snaplen field represents the length of the captured
        //  portion of the packet. If snaplen is less than length, it means that only a portion of the
        //  packet was captured, and the rest was truncated.

        println!("[{:?}] ---> [{:?}] - Protocol: {} \n", src_ip, dst_ip, protocol);
    };

    // cap.for_each(None, pkt_handler).unwrap();
    
    cap.for_each(Some(100), pkt_handler).unwrap();
    println!("Received {} packets", cap.stats().unwrap().received);
    println!("Dropped {} packets", cap.stats().unwrap().dropped);
    println!("Interface dropped {} packets", cap.stats().unwrap().if_dropped);
}
