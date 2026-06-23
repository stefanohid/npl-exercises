use std::net::{UdpSocket, SocketAddr};
// use std::net::ToSocketAddrs;
use clap::Parser;


#[derive(Parser, Debug)]
struct Cli {
    /// Server host
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
    /// Server port
    #[arg(short, long, default_value_t = 10000)]
    port: u16,
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}



fn main () {
    let cli = Cli::parse();

    let srv_sock: SocketAddr = format!("{}:{}", cli.host, cli.port).parse().unwrap();

//    In case we want to support hostnames instead of IP addresses, we can use the following code to resolve the server address
//    let srv_sock: SocketAddr = format!("{}:{}", cli.host, cli.port)
//        .to_socket_addrs().unwrap()
//        .find(|addr| addr.is_ipv4())
//        .ok_or_else(|| std::io::Error::new(
//            std::io::ErrorKind::AddrNotAvailable,
//            "no IPv4 address found",
//        )).unwrap();
    let sock = UdpSocket::bind(srv_sock).expect("Unable to open socket");

    verbose_log(cli.verbose, 1, format!("UDP server started on {}:{}", sock.local_addr().unwrap().ip(), cli.port));

    loop {
        let mut buffer = [0_u8;1500];
        let (nbytes, clt) = sock.recv_from(&mut buffer).unwrap();

        let message = format!("Received {} bytes from {}:{}", nbytes, clt.ip(), clt.port());
        verbose_log(cli.verbose, 2, message);

        let s = str::from_utf8(&buffer[..nbytes]).unwrap();
        let response = s.to_uppercase();

        let _ = sock.send_to(response.as_bytes(), clt).expect("Can't send response");
    }
}


fn verbose_log(verbose: u8, level: u8, message: String) {
    if verbose >= level {
        eprintln!("{message}");
    } 
}