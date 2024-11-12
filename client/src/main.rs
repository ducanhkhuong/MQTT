//client

use std::io::{self, BufRead, Write};
use std::net::TcpStream;
use std::thread;
use std::io::Read;



fn main() {
    //tao ket noi toi server
    match TcpStream::connect("127.0.0.1:8080") {
        Ok(mut stream) => {
            println!("[Client] ---> [Server]   : OK");

            //nhap trang thai cua client
            let stdin = io::stdin();
            let mut input = String::new();
            println!("1 : Subscribe");
            println!("2 : Publish");
            stdin.lock().read_line(&mut input).expect("Failed to read line");
            let choice = input.trim();

            //xu ly logic 1 or 2
            match choice {
                "1" => {
                    //nhap ten topic muon dang ki
                    println!("Topic to subscribe:");
                    input.clear();
                    stdin.lock().read_line(&mut input).expect("Failed to read line");
                    let topic = input.trim().to_string();

                    //fomat dinh dang va tien hanh gui yeu cau
                    let subscribe_request = format!("SUBSCRIBE {}", topic);
                    stream.write_all(subscribe_request.as_bytes()).expect("Failed to send subscribe request");

                    //neu co topic cung loai thi gui dinh messager va topic
                    //gui subscript
                    let mut stream_clone = stream.try_clone().expect("Failed to clone stream");
                    thread::spawn(move || {
                        loop {
                            let mut buffer = [0; 1024];
                            match stream_clone.read(&mut buffer) {
                                Ok(n) => {
                                    if n == 0 {
                                        break;
                                    }
                                    let message = String::from_utf8_lossy(&buffer[..n]);
                                    println!("[Server] ---> [Client] - message : {}", message);
                                }
                                Err(e) => {
                                    eprintln!("Error : {}", e);
                                    break;
                                }
                            }
                        }
                    }).join().expect("Thread join failed");
                }
                "2" => {
                    loop{
                        println!("Topic to publish:");
                        input.clear();
                        stdin.lock().read_line(&mut input).expect("Failed to read line");
                        let topic = input.trim().to_string();

                        println!("Message to publish:");
                        input.clear();
                        stdin.lock().read_line(&mut input).expect("Failed to read line");
                        let message = input.trim().to_string();
                        
                        //gui publish
                        let publish_request = format!("PUBLISH {} {}", topic, message);
                        stream.write_all(publish_request.as_bytes()).expect("Failed to send publish request");
                    }
                }
                _ => {
                    println!("Invalid choice");
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to connect: {}", e);
        }
    }
}

