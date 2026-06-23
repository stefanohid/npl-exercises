use pcap::Capture;

/// Test it with a dump as such:
/// sudo tcpdump -i eth0 ether src 02:00:00:00:00:01 -XX
fn main() {
    let iface = "eth0";

    let mut cap = Capture::from_device(iface)
        .expect("Cannot create capture from device")
        .promisc(true)
        .timeout(1000)
        .open()
        .expect("Cant open capture on device");

    let dst_mac = [0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
    let src_mac = [0x02, 0x00, 0x00, 0x00, 0x00, 0x01];
    let ethertype = [0x08, 0x00]; // IPv4
    let payload = b"Hello, pcap";

    loop {
        let mut packet = Vec::with_capacity(14 + payload.len());
        packet.extend_from_slice(&dst_mac);
        packet.extend_from_slice(&src_mac);
        packet.extend_from_slice(&ethertype);
        packet.extend_from_slice(payload);

        cap.sendpacket(packet).expect("Failed to send packet");
        println!("Packet sent");

        std::thread::sleep(std::time::Duration::from_secs(1));
        // This method allows to perform a replay by taking a look at the inter temp between two packets, 
        // and then sleeping for that amount of time before sending the next packet.
        // 
        // In order to keep the bitrate fixed, we would need to tweak the sleep time based on the size of 
        // the packet, and the desired bitrate. For example, if we want to send packets at a bitrate of 1 
        // Mbps, and our packet size is 100 bytes (800 bits), we would need to send 125 packets per second 
        // (1,000,000 bits per second / 800 bits per packet). This means we would need to sleep for 8 
        // milliseconds between each packet (1000 ms / 125 packets per second).
    }
}