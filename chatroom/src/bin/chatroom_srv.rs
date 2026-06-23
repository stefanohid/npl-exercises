use std::net::{SocketAddr,UdpSocket};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Serialize, Deserialize, Debug)]
struct Msg {
    msg: String,
    username: String
}

fn main() {
    let srv_sock: SocketAddr = format!("0.0.0.0:8080").parse().unwrap();
    let sock = UdpSocket::bind(srv_sock).expect("Cannot bind!");

    let mut user_map: HashMap<String, SocketAddr> = HashMap::new();

    loop {
        let mut buf = [0_u8;1500];
        let (nbytes, clt) = sock.recv_from(&mut buf).unwrap();
        
        let received = match std::str::from_utf8(&buf[..nbytes]) {
            Ok(text) => text.trim(),
            Err(error) => {
                eprintln!("Invalid UTF-8 from {clt}: {error}");
                continue;
            }
        };

        let message: Msg = serde_json::from_str(received).unwrap();

        let raw_message = message.msg.as_str();
        let user = &message.username.clone();
        println!("Received a message: {}", raw_message);

        match raw_message.split_once('|') {
            Some((command, arguments)) => {
                match command {
                    "JOIN" => {
                        if None == user_map.get(user) {
                            *user_map.entry(message.username).or_insert(format!("0.0.0.0:0").parse().unwrap()) = clt;
                            println!("Inserted user {} into map", user);
                            send_broadcast(&user_map, user, &sock, format!("User {} joined the chatroom!", user));
                        } else {
                            sock.send_to(format!("ERROR|The username {} already exists!", message.username).as_bytes(), clt)
                                .expect("Could not send message to client!");
                        }
                    }
                    "QUIT" => {
                        if None == user_map.get(&message.username) {
                            sock.send_to(format!("ERROR|Username {} not in chatroom!", message.username).as_bytes(), clt)
                                .expect("Could not send message to client!");
                        } else {
                            user_map.remove(&message.username);
                            send_broadcast(&user_map, user, &sock, format!("User {} left the chatroom!", user));
                        }
                    }
                    "SEND" => {
                        match arguments.split_once('|') {
                            Some((sender, text)) => {
                                send_broadcast(
                                    &user_map,
                                    user,
                                    &sock,
                                    format!("<{}> {}", sender, text),
                                );
                            }
                            None => {
                                sock.send_to(b"ERROR|Invalid public message", clt)
                                    .expect("Could not send message to client!");
                            }
                        }
                    }
                    "PRIV" => {
                        match arguments.split_once('|') {
                            Some((target, text)) => {
                                let target_address = user_map.get(target);
                                if target_address.is_none() {
                                    sock.send_to(format!("ERROR|Username {} not in chatroom!", target).as_bytes(), clt)
                                        .expect("Could not send message to client!");
                                } else {
                                    sock.send_to(
                                        format!("SEND|[Private from {}] {}", message.username, text).as_bytes(),
                                        target_address.unwrap(),
                                    )
                                        .expect("Could not send message to client!");
                                }
                            }
                            None => {
                                sock.send_to(b"ERROR|Invalid private message", clt)
                                    .expect("Could not send message to client!");
                            }
                        }
                    }
                    _ => {
                        println!("Unknown command: {}", command);
                        sock.send_to("ERROR|Unknown command".as_bytes(), clt)
                            .expect("Could not send message to client!");
                    }
                }
            }
            None => {
                sock.send_to("ERROR|Invalid message: missing |".as_bytes(), clt)
                            .expect("Could not send message to client!");
            }
        }

    }
    
}

fn send_broadcast(user_map: &HashMap<String, SocketAddr>, username: &String, sock: &UdpSocket, broadcast: String) {
    for user in user_map.iter() {
        if user.0.as_str() != username {
            sock.send_to( format!("SEND|{}", broadcast).as_bytes(), user.1)
                .expect("Could not send message to client!");
        }
    }
}
