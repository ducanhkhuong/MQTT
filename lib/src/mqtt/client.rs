pub mod client{
    use std::io::{self, BufRead, Read, Write};
    use std::net::TcpStream;
    use std::thread;

    #[derive(Debug, PartialEq, Eq)]
    pub enum Ex {
        Empty,
        Exit,
        Mqtt,
    }

    pub enum Command {
        Subscribe { topic: String },
        Publish { topic: String, message: String },
        Help,
    }

    pub struct Client {
        stream: TcpStream,
    }

    impl Client {
        pub fn new(addr: &str) -> Result<Client, String> {
            match TcpStream::connect(addr) {
                Ok(stream) => {
                    println!("[Client] ---> [Broker] : OK");
                    Ok(Client { stream })
                }
                Err(e) => Err(format!("Failed to connect: {}", e)),
            }
        }

        pub fn start(&mut self) {
            let stdin = io::stdin();
            let mut input = String::new();

            let mut stream_clone = self.stream.try_clone().expect("Failed to clone stream");

            //luồng đọc dữ liệu từ broker nếu là client sub
            thread::spawn(move || loop {
                let mut buffer = [0; 1024];
                match stream_clone.read(&mut buffer) {
                    Ok(n) if n == 0 => break,
                    Ok(n) => {
                        let message = String::from_utf8_lossy(&buffer[..n]);

                        let mut parts = message.splitn(2, ' ');
                        let topic = parts.next().unwrap_or("Unknown Topic");
                        let message = parts.next().unwrap_or("Empty Message");

                        println!("[Broker] ---> [Client] with topic '{}' : {}", topic, message);
                    }
                    Err(e) => {
                        eprintln!("Error reading from stream: {}", e);
                        break;
                    }
                }
            });

            //luồng chính
            Self::help();

            loop {
                input.clear();
                stdin.lock().read_line(&mut input).expect("Failed to read line");
                let command = input.trim();

                if command.is_empty() {
                    println!("warning: empty ---> run : mqtt HELP");
                    continue;
                }

                match Self::parse_command(command) {
                    Ok(Ex::Empty) => {
                        println!("warning: Invalid command ---> run : mqtt HELP");
                        continue;
                    }
                    Ok(Ex::Exit) => {
                        break;
                    }
                    Ok(Ex::Mqtt) => {
                        self.handle_command(command);
                    }
                    Err(e) => println!("{}", e),
                }
            }
        }



        fn parse_command(command: &str) -> Result<Ex, String> {
            let parts: Vec<&str> = command.split_whitespace().collect();

            if parts.len() == 1 {
                return Ok(Ex::Empty);
            }
            if parts.len() >= 2 && parts[0].to_lowercase() == "mqtt" {
                return Ok(Ex::Mqtt);
            } 
            else if parts.len() == 1 && (parts[0].eq_ignore_ascii_case("exit")) {
                Ok(Ex::Exit)
            } else {
                Err("warning: Invalid command ---> run : mqtt HELP".to_string())
            }
        }



        fn handle_command(&mut self, command: &str) {
            let parts: Vec<&str> = command.split_whitespace().collect();
            match parts[1].trim(){
                "SUBSCRIBE" => {
                    let topic = parts[2].to_string();
                    let subscribe_request = format!("SUBSCRIBE {}", topic);
                    self.stream
                        .write_all(subscribe_request.as_bytes())
                        .expect("Failed to send subscribe request");
                }
                "PUBLISH" => {
                    let topic = parts[2].to_string();
                    let message = parts[3].to_string();
                    let publish_request = format!("PUBLISH {} {}", topic, message);
                    self.stream
                        .write_all(publish_request.as_bytes())
                        .expect("Failed to send publish request");
                    println!("[Client] ---> [Broker] with topic '{}' - message '{}' : send OK ", topic, message);
                }
                "HELP" => {
                    Self::help();
                }
                _ => {
                    println!("warning: Invalid command ---> run : mqtt HELP");
                }
            }
        }

        fn help() {
            println!("Enter command : ");
            println!("                mqtt SUBSCRIBE topic        ");
            println!("                mqtt PUBLISH   topic message");
            println!("                mqtt HELP                 ");
        }
    }
}
