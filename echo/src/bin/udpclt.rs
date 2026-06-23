use std::net::{SocketAddr, UdpSocket};
// use std::net::ToSocketAddrs;
use clap::Parser;


#[derive(Parser, Debug)]
//Defines the command line arguments for the UDP client using the clap crate. 
//The struct Cli will be automatically populated with the values provided by the user when running the program, 
//and we can access these values in the main function to configure the client's behavior.
//Using this as a default we get -h or --help for free, and it will show the user the available options and their descriptions. 
struct Cli {
    /// Server host 
    #[arg(short, long, default_value = "127.0.0.1")] //long: --server, short: -s
    server: String,
    /// Server port
    #[arg(short, long, default_value_t = 10000)] //_t means that the default value is of the same type as the field, in this case u16, otherwise string
    port: u16,
    // Verbose mode (-v, -vv, -vvv, etc.) - This argument allows the user to specify the verbosity level of the client's output. 
    // Each occurrence of -v increases the verbosity level by 1, so -v sets verbose to 1, -vv sets it to 2,
    // and so on. The verbose_log function can then use this value to determine which messages to print
    // based on their importance level.
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}


fn main() {
    let cli = Cli::parse();
    // let clt_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED),0);

    let clt_addr: SocketAddr = "0.0.0.0:0".parse().unwrap();

    let srv_sock: SocketAddr = format!("{}:{}", cli.server, cli.port).parse().unwrap();

//    In case we want to support hostnames instead of IP addresses, we can use the following code to resolve the server address
//    let srv_sock: SocketAddr = format!("{}:{}", cli.server, cli.port)
//        .to_socket_addrs().unwrap()
//        .find(|addr| addr.is_ipv4())
//        .ok_or_else(|| std::io::Error::new(
//            std::io::ErrorKind::AddrNotAvailable,
//            "no IPv4 address found",
//        )).unwrap();

    let sock = UdpSocket::bind(clt_addr).unwrap();
    let local_socket = sock.local_addr().unwrap();
    verbose_log(cli.verbose, 1, format!("UDP client started on {}:{}", local_socket.ip(), local_socket.port()));

    
    loop {
        let mut input = String::new();
        let nn = std::io::stdin().read_line(&mut input).unwrap();

        if nn == 0 {
            verbose_log(cli.verbose, 1, "EOF received, exiting".to_string()); // EOF: CTRL+D
            break;
        }

        let _ = sock.send_to(input.as_bytes(), srv_sock).unwrap();
        
        let mut buffer = [0_u8;1500];
        let (nbytes, _) = sock.recv_from(&mut buffer).unwrap();

        if nbytes == 0 {
            verbose_log(cli.verbose, 1, "Server closed connection".to_string());
            break;
        }
        
        let response = std::str::from_utf8(&buffer[..nbytes]).unwrap();
        print!("{}",response);
//      Note: Could have used print!(), but in this way the output may not appear immediately unless stdout is flushed, especially when printing prompts or progress text.
    }
}

fn verbose_log(verbose: u8, level: u8, message: String) {
    if verbose >= level {
        eprintln!("{message}");
    }
}