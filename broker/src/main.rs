//broker

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;


type Topic = String;
type Subscribers = Arc<Mutex<HashMap<Topic, Vec<TcpStream>>>>;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").expect("Failed");
    let subscribers: Subscribers = Arc::new(Mutex::new(HashMap::new()));

    println!("[Server] : OK");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let subscribers = Arc::clone(&subscribers);
                thread::spawn(move || {
                    handle_client(stream, subscribers);
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
}

fn handle_client(mut stream: TcpStream, subscribers: Subscribers) {
    let mut buffer = [0; 1024];
    while let Ok(n) = stream.read(&mut buffer) {
        if n == 0 {
            return;
        }

        //chuyen bo dem tu 1024 byte sang chuỗi string
        let request = String::from_utf8_lossy(&buffer[..n]);


        //them gia tri vao vector và chia phần nếu hơn 2 phần thì tiếp tục
        let parts: Vec<&str> = request.splitn(3, ' ').collect();

        if parts.len() < 2 {
            eprintln!("Invalid request: {}", request);
            continue;
        }

        //logic vecto
        //part.[0,1,2,]=>[pub,topic,messager]
        //part.[0,1]=>[sub,topic]
        match parts[0] {
            "SUBSCRIBE" => {
                let topic = parts[1].to_string();
                let mut subscribers = subscribers.lock().unwrap();
                let entry = subscribers.entry(topic.clone()).or_insert_with(Vec::new);
                entry.push(stream.try_clone().expect("Failed to clone stream"));
                println!("SUBSCRIBE : [Client] ---> [Server] - Topic : {}", topic);
            }
            "PUBLISH" => {
                if parts.len() < 3 {
                    eprintln!("Invalid publish request: {}", request);
                    continue;
                }
                let topic = parts[1].to_string();
                let message = parts[2].to_string();
                let subscribers = subscribers.lock().unwrap();
                if let Some(subs) = subscribers.get(&topic) {
                    for mut sub in subs {
                        if let Err(e) = sub.write_all(message.as_bytes()) {
                            eprintln!("Error sending message to subscriber: {}", e);
                        }
                    }
                }
                println!("PUBLISH : [Client] ---> [Server] - Topic:{},Message:{}",topic,message);
            }
            _ => {
                eprintln!("Unknown command: {}", parts[0]);
            }
        }
    }
}
