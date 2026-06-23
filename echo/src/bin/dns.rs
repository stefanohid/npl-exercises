use clap::Parser;
use std::net::ToSocketAddrs;

#[derive(Parser, Debug)]
struct Cli {
    /// Server host 
    #[arg(default_value = "www.google.com")]
    host: String,
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
    #[arg(short = '4', long)]
    ipv4: bool,
    #[arg(short = '6', long)]
    ipv6: bool,
}

fn main() {
    let cli = Cli::parse();
    let socket_addr = format!("{}:0", cli.host);
    ToSocketAddrs::to_socket_addrs(&socket_addr)
    .unwrap()
    .for_each(|addr| {
        if cli.ipv4 && !addr.is_ipv4() {
            return;
        }

        if cli.ipv6 && !addr.is_ipv6() {
            return;
        }

        verbose_log(cli.verbose, 1, format!("Resolved address: {}", addr));
    });

}

fn verbose_log(verbose: u8, level: u8, message: String) {
    if verbose >= level {
        eprintln!("{message}");
    }
}