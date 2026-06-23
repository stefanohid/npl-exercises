use std::net::{UdpSocket, SocketAddr};

use clap::Parser;


#[derive(Parser, Debug)]
struct Cli {
    /// Server host
    #[arg(long, default_value = "0.0.0.0")]
    host: String,
    /// Server port
    #[arg(short, long, default_value_t = 10000)]
    port: u16,
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
    /// News
    #[arg(long, default_value = "Hello World!")]
    news: String,
    /// Interval in seconds
    #[arg(short, long, default_value_t = 3)]
    interval: u64,
}

/// Start with: ./target/debug/broadcastupdsrv --host 127.0.0.1 --port 10000 --news "Test" -vvv
fn main() {
    let cli  = Cli::parse();
    let srv_sock: SocketAddr = format!("{}:0", cli.host).parse().unwrap();

    let sock = UdpSocket::bind(srv_sock).expect("Unable to open socket");
    sock.set_broadcast(true).expect("Could not enable broadcast");
    let broadcast = format!("{}", cli.news);
    verbose_log(cli.verbose, 1, format!("UDP server started on {}:{}", sock.local_addr().unwrap().ip(), cli.port));

    loop {
        let _ = sock.send_to(broadcast.as_bytes(), format!("255.255.255.255:{}", cli.port)).expect("Can't send response");
        verbose_log(cli.verbose, 1, "Sent broadcast message".to_string());
        std::thread::sleep(std::time::Duration::from_secs(cli.interval));
    }
}

fn verbose_log(verbose: u8, level: u8, message: String) {
    if verbose >= level {
        eprintln!("{message}");
    }
}