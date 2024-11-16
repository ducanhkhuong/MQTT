pub mod client{
    use std::io::{self, Write, Read,BufRead};
    use std::net::TcpStream;
    use std::thread;

    pub struct Client {
        stream: TcpStream,
    }
    
    impl Client {
        pub fn new(addr: &str) -> Result<Client, String> {
            match TcpStream::connect(addr) {
                Ok(stream) => {
                    println!("[Client] ---> [Broker]  : OK");
                    Ok(Client { stream })
                }
                Err(e) => Err(format!("Failed to connect: {}", e)),
            }
        }
    
        pub fn start(&mut self, choice: &str) {
            match choice {
                "1" => {
                    self.subscribe();
                }
                "2" => {
                    self.publish();
                }
                _ => {
                    println!("Invalid choice");
                }
            }
        }
    
        
        fn start_receiving(&mut self) {
            let mut stream_clone = self.stream.try_clone().expect("Failed to clone stream");
            thread::spawn(move || {
                loop {
                    let mut buffer = [0; 1024];
                    match stream_clone.read(&mut buffer) {
                        Ok(n) => {
                            if n == 0 {
                                break;
                            }
                            let message = String::from_utf8_lossy(&buffer[..n]);
                            println!("[Broker] ---> [Client] - message : {}", message);
                        }
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            break;
                        }
                    }
                }
            });
        }
    
        fn subscribe(&mut self) {
            let stdin = io::stdin();
            let mut input = String::new();
    
            println!("Topic to subscribe:");
            input.clear();
            stdin.lock().read_line(&mut input).expect("Failed to read line");
            let topic = input.trim().to_string();
    
            let subscribe_request = format!("SUBSCRIBE {}", topic);
            self.stream.write_all(subscribe_request.as_bytes()).expect("Failed to send subscribe request");
    
            self.start_receiving();
        }
    
        fn publish(&mut self) {
            let stdin = io::stdin();
            let mut input = String::new();
    
            loop {
                println!("Topic to publish:");
                input.clear();
                stdin.lock().read_line(&mut input).expect("Failed to read line");
                let topic = input.trim().to_string();
    
                println!("Message to publish:");
                input.clear();
                stdin.lock().read_line(&mut input).expect("Failed to read line");
                let message = input.trim().to_string();
    
                let publish_request = format!("PUBLISH {} {}", topic, message);
                self.stream.write_all(publish_request.as_bytes()).expect("Failed to send publish request");
            }
        }
    }    
}


