use clap::Parser;
use std::net::{UdpSocket, SocketAddr};

#[derive(Parser, Debug)]
struct Cli {
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

/// Start with: ./target/debug/broadcastupdclt --port 10000 -vvv
fn main() {
    let cli = Cli::parse();
    let srv_sock: SocketAddr = format!("0.0.0.0:{}", cli.port).parse().unwrap();
    let sock = UdpSocket::bind(srv_sock).expect("Unable to open socket");
    verbose_log(cli.verbose, 1, format!("UDP client started on {}:{}", sock.local_addr().unwrap().ip(), sock.local_addr().unwrap().port()));

    loop {
    let mut buffer = [0_u8; 1500];

    let (nbytes, sender) = sock.recv_from(&mut buffer).unwrap();

    verbose_log(
        cli.verbose,
        1,
        format!("Received {} bytes from {}", nbytes, sender),
    );

    let news = std::str::from_utf8(&buffer[..nbytes]).unwrap();

    println!("News from {}: {}", sender, news);
}

}

fn verbose_log(verbose: u8, level: u8, message: String) {
    if verbose >= level {
        eprintln!("{message}");
    }
}