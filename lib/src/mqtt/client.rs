pub mod client {
    use std::io::{self, BufRead, Read, Write};
    use std::net::TcpStream;
    use std::thread;

    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub enum Qos {
        Low,
        Medium,
        High,
    }

    #[derive(Debug, PartialEq, Eq)]
    pub enum Ex {
        Empty,
        Exit,
        Mqtt,
    }

    pub enum Command {
        Subscribe { topic: String, qos: Qos },
        Publish   { topic: String, message: String, qos: Qos },
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

            //nhận dữ liệu tử broker nếu là client subscribe
            thread::spawn(move || loop {
                let mut buffer = [0; 1024];
                match stream_clone.read(&mut buffer) {
                    Ok(n) if n == 0 => break,
                    Ok(n) => {
                        let message = String::from_utf8_lossy(&buffer[..n]);

                        let mut parts = message.splitn(2, ' ');
                        let topic = parts.next().unwrap_or("Unknown Topic");
                        let message = parts.next().unwrap_or("Empty Message");

                        println!("[Broker] ---> [Client] : topic '{}' : {}", topic, message);
                    }
                    Err(e) => {
                        eprintln!("Error reading from stream: {}", e);
                        break;
                    }
                }
            });

            // Main loop            
            println!("Welcome mqtt:");
            Self::help();

            loop {
                input.clear();
                stdin.lock().read_line(&mut input).expect("Failed to read line");
                let command = input.trim();

                if command.is_empty() {
                    continue;
                }

                match Self::parse_command(command) {
                    Ok(Ex::Empty) => {
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
                println!("warning: Invalid command ---> run : mqtt -H");
                return Ok(Ex::Empty);
            }
            if parts.len() >= 2 && parts[0].to_lowercase() == "mqtt" {
                return Ok(Ex::Mqtt);
            } else if parts.len() == 1 && (parts[0].eq_ignore_ascii_case("exit")) {
                Ok(Ex::Exit)
            } else {
                Err("warning: Invalid command ---> run : mqtt -H".to_string())
            }
        }

        fn handle_command(&mut self, command: &str) {
            let parts: Vec<&str> = command.split_whitespace().collect();

            let cmd = match parts[1]{
                "SUBSCRIBE"|"--S" => {
                    if parts.len() < 4 {
                        println!("warning: Invalid command ---> run : mqtt -H");
                        return;
                    }
                    let topic = parts[2].to_string();
                    let qos = self.parse_qos(parts[3]);
                    Command::Subscribe { topic, qos }
                }
                "PUBLISH"|"--P" => {
                    if parts.len() < 5 {
                        println!("warning: Invalid command ---> run : mqtt -H");
                        return;
                    }
                    let topic = parts[2].to_string();
                    let message = parts[3].to_string();
                    let qos = self.parse_qos(parts[4]);
                    Command::Publish { topic, message, qos }
                }
                "HELP"|"--H" => {
                    Command::Help
                },
                _ => {
                    println!("warning: Invalid command ---> run : mqtt -H");
                    return;
                }
            };

            match cmd {
                Command::Subscribe { topic, qos } => {

                    let subscribe_request = format!("SUBSCRIBE {} {}", topic, qos as u8);
                    self.stream
                        .write_all(subscribe_request.as_bytes())
                        .expect("Failed to send subscribe request");
                    println!("[Client] ---> [Broker] : SUBSCRIBE to '{}' with QoS '{:?}'", topic, qos);

                }
                Command::Publish { topic, message, qos } => {

                    let publish_request = format!("PUBLISH {} {} {}", topic, message, qos as u8);
                    self.stream
                        .write_all(publish_request.as_bytes())
                        .expect("Failed to send publish request");
                    println!("[Client] ---> [Broker] : PUBLISH to '{}' with message '{}' and QoS '{:?}'", topic, message, qos);

                }

                Command::Help => Self::help(),
            }
        }

        fn help() {
            println!("                                                 ");
            println!("[Usage]:        mqtt [OPTIONS] [COMMAND] [QOS]   ");
            println!("                                                 ");    
            println!("[OPTION] :                                       ");
            println!("                  --S : SUBSCRIBE                ");
            println!("                  --P : PUBLISH                  ");
            println!("                  --H : HELP                     ");
            println!("[COMMAND]:");
            println!("             topic   : Specify topic             ");
            println!("             message : Specify message           ");
            println!("[QOS]    :");
            println!("                   0 : QoS 0 - At most once      ");
            println!("                   1 : QoS 1 - At least once     ");
            println!("                   2 : QoS 2 - Exactly once      ");
            println!("[EXAMPLE]:");
            println!("                   mqtt --S topic qos            ");
            println!("                   mqtt --P topic message qos    ");
            println!("                   mqtt --H                      ");
        }

        fn parse_qos(&self, qos_str: &str) -> Qos {
            match qos_str.to_lowercase().as_str() {
                "0" => Qos::Low,
                "1" => Qos::Medium,
                "2" => Qos::High,
                _ =>   Qos::Low,
            }
        }
    }
}
