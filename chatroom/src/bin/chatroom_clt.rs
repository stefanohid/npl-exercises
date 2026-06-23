use serde::{Deserialize, Serialize};
use std::io;
use std::net::{SocketAddr, UdpSocket};

#[derive(Serialize, Deserialize, Debug)]
struct Msg {
    msg: String,
    username: String,
}

fn main() {
    let clt_addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let srv_addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let clt_sock = UdpSocket::bind(clt_addr).expect("Could not bind!");
    clt_sock.connect(srv_addr)
        .expect("Could not connect UDP socket");

    let username = loop {
        println!("Enter your username:");

        let mut username = String::new();
        io::stdin()
            .read_line(&mut username)
            .expect("Could not read username");

        let username = username.trim().to_string();

        if username.is_empty() || username.contains('|') {
            println!("Invalid username. Try again.");
            continue;
        }

        break username;
    };

    println!("Welcome, {}!", username);

    let join_packet = Msg {
        msg: format!("JOIN|{}", username),
        username: username.clone(),
    };
    send_message(&clt_sock, &join_packet);

    let receiver_socket = clt_sock
        .try_clone()
        .expect("Could not clone client socket");

    std::thread::spawn(move || loop {
        let mut buf = [0_u8; 1500];

        match receiver_socket.recv(&mut buf) {
            Ok(nbytes) => {
                let received = String::from_utf8_lossy(&buf[..nbytes]);

                match received.split_once('|') {
                    Some(("SEND", text)) => println!("{}", text),
                    Some(("ERROR", text)) => eprintln!("Error: {}", text),
                    Some((kind, text)) => {
                        eprintln!("Unknown server message '{}': {}", kind, text);
                    }
                    None => eprintln!("Malformed server message: {}", received),
                }
            }
            Err(error) => {
                eprintln!("Could not receive message: {}", error);
                break;
            }
        }
    });

    loop {
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Could not read input");

        let input = input.trim_end_matches(['\r', '\n']);

        match input {
            "/quit" => {
                let packet = Msg {
                    msg: format!("QUIT|{}", username),
                    username: username.clone(),
                };

                send_message(&clt_sock, &packet);
                break;
            }

            command if command.starts_with("/msg ") => {
                let arguments = command.strip_prefix("/msg ").unwrap();

                match arguments.split_once(' ') {
                    Some((target, text)) if !target.is_empty() && !text.is_empty() => {
                        let packet = Msg {
                            msg: format!("PRIV|{}|{}", target, text),
                            username: username.clone(),
                        };

                        send_message(&clt_sock, &packet);
                    }

                    _ => {
                        println!("Usage: /msg <username> <message>");
                    }
                }
            }

            "" => {}

            text => {
                let packet = Msg {
                    msg: format!("SEND|{}|{}", username, text),
                    username: username.clone(),
                };

                send_message(&clt_sock, &packet);
            }
        }
    }
}

fn send_message(socket: &UdpSocket, message: &Msg) {
    let json = serde_json::to_string(message)
        .expect("Could not serialize message");

    socket.send(json.as_bytes())
        .expect("Could not send message");
}
