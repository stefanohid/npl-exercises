use clap::Parser;
use serde::{Serialize, Deserialize};
use std::{net::{SocketAddr, UdpSocket}};
use rand::seq::SliceRandom;
use std::io::BufRead;

#[derive(Parser, Debug)]
struct Cli {
    /// Server host
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
    /// Server port
    #[arg(long, default_value_t = 10000)]
    port: u16,
    /// Verbose mode
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[derive(Serialize, Deserialize, Debug)]
enum Fortune {
    MessageType(String),
    Text(String),
}

/// Listen for client requests. When a request is received, it prepares a fortune message and sends it back 
///  to the client. Again, the message is formatted as JSON with type = REP and the text field filled with the
///  fortune sentence.
/// The application is supposed to run over UDP by using an arbitrary port (e.g. 12000).
/// 
/// echo '{"MessageType":"REQ"}' | nc -u -w1 127.0.0.1 10000
fn main () {
    let cli = Cli::parse();
    let srv_sock: SocketAddr = format!("{}:{}", cli.host, cli.port).parse().unwrap();
    let sock = UdpSocket::bind(srv_sock).expect("Could not bind sock");
    verbose_log(cli.verbose, 1, format!("UDP server started on {}:{}", sock.local_addr().unwrap().ip(), cli.port));

    loop {
        let mut buffer = [0_u8;1500];
        let (nbytes, clt) = sock.recv_from(&mut buffer).unwrap();
        let received = std::str::from_utf8(&buffer[..nbytes]).unwrap();
        let msg: Fortune = serde_json::from_str(received).unwrap();

        match msg {
            Fortune::MessageType(message_type) => {
                if message_type == "REQ" {
                    println!("Client requested a fortune!");
                } else {
                    panic!("Unknown message type!");
                }
            }

            Fortune::Text(text) => {
                panic!("Did not expect text from client, but got: {text}");
            }
        }

        let input = std::fs::File::open("/home/stefa/npl-projects/fortune_22_07_18/src/fortunes.txt")
            .expect("Could not open file");
        let reader = std::io::BufReader::new(input);

        let fortunes: Vec<String> = reader
            .lines()
            .map(|line| line.expect("Could not read line"))
            .collect();

        let fortune = fortunes
            .choose(&mut rand::thread_rng())
            .expect("No fortunes available");

        let response: Fortune = Fortune::Text(format!("{fortune}"));
        let json_response = serde_json::to_string(&response).unwrap();

        let _ = sock.send_to(json_response.as_bytes(), clt).expect("Can't send response");
        verbose_log(cli.verbose, 1, format!("Sent fortune to client: {fortune}"));

    }
}

fn verbose_log(verbose: u8, level: u8, message: String) {
    if verbose >= level {
        eprintln!("{message}");
    } 
}