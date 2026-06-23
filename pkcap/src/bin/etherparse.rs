use etherparse;
use pcap::Capture;
fn main() {
    let mut cap = Capture::from_device("eth0")
    .unwrap()
    .immediate_mode(true)
    .promisc(true)
    .open()
    .unwrap();

    // This is the raw slicing. It contains a struct of 4 options, one for each layer (link, network, 
    //  transport, payload). Each option is either Some or None.
    // while let Ok(packet) = cap.next_packet() {
    //     println!("Got a packet! {:?}", &packet.data[12..14]);
    //     match etherparse::SlicedPacket::from_ethernet(&packet.data) {
    //         Ok(sliced) => {
    //             println!("Parsed packet: {:?}", sliced);
    //         }
    //         Err(e) => {
    //             println!("Failed to parse packet: {:?}", e);
    //         }
    //     }
    // }

    // This, instead, is header deserialization.
    while let Ok(packet) = cap.next_packet() {
        println!("Got a packet! {:?}", &packet.data[12..14]);
        let h = etherparse::PacketHeaders::from_ethernet_slice(&packet.data).unwrap();
        
        match h.net {
                    Some(etherparse::NetHeaders::Ipv4(header, _)) => {
                        println!("IPv4 packet: {:?}", header);
                    }
                    Some(etherparse::NetHeaders::Ipv6(header, _)) => {
                        println!("IPv6 packet: {:?}", header);
                    }
                    _ => {
                        println!("Not an IP packet");
                    }
        }

    }

}