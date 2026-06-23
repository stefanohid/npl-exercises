use std::net::{SocketAddr, TcpListener};
use std::io::{BufRead, BufReader, Write};
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
    let sock = TcpListener::bind(srv_sock).expect("Unable to open socket");

    verbose_log(cli.verbose, 1, format!("TCP server started on {}:{}", sock.local_addr().unwrap().ip(), cli.port));

    for connection in sock.incoming() {
        match connection {
            Ok(stream) => {
                verbose_log(cli.verbose, 1, format!("Connected to Client {}:{}", stream.peer_addr().unwrap().ip(), stream.peer_addr().unwrap().port()));
                handle_client(stream, cli.verbose);
            }
            Err(_) => {
                eprintln!("Error accepting connection");
                continue;
            }
        }
    }

    // loop {
    //     let (stream, clt) = sock.accept().unwrap();
    //     verbose_log(cli.verbose, 1, format!("Conenected to Client {}:{}", clt.ip(), clt.port()));
    //     let cloned_stream = stream.try_clone().expect("Failed to clone stream");
    //     let mut writer_stream = stream;

    //     let reader = BufReader::new(cloned_stream);
    //     for line in reader.lines() {
    //         let line: String = line.unwrap();
    //         verbose_log(cli.verbose, 2, format!("Received from {}: {} bytes", clt, line.len()));
    //         let response = line.to_uppercase();
    //         writer_stream.write_all(response.as_bytes()).expect("Can't send response");
    //     }

    //     // let nbytes = stream.read(&mut buffer).unwrap();
    //     // let s = str::from_utf8(&buffer[..nbytes]).unwrap();
    //     // let response = s.to_uppercase();

    //     // let _ = stream.write_all(response.as_bytes()).expect("Can't send response");
    //     //we use write_all instead of write to ensure that all bytes are sent, since write may not send all bytes in one call.
    // }
}


fn verbose_log(verbose: u8, level: u8, message: String) {
    if verbose >= level {
        eprintln!("{message}");
    }
}

fn handle_client(stream: std::net::TcpStream, verbose: u8) {
    let clt = stream.peer_addr().unwrap();
    let cloned_stream = stream.try_clone().expect("Failed to clone stream");
    let mut writer_stream = stream;

    let reader = BufReader::new(cloned_stream);
    for line in reader.lines() {
        let line: String = line.unwrap();
        verbose_log(verbose, 2, format!("Received from {}: {} bytes", clt, line.len()));
        let response = line.to_uppercase();
        writer_stream.write_all(response.as_bytes()).expect("Can't send response");
    }
}