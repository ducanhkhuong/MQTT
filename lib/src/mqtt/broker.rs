pub mod broker {
    use std::collections::HashMap;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream};
    use tokio::sync::mpsc;
    use tokio::sync::mpsc::UnboundedSender;

    #[derive(Debug, PartialEq, Copy, Clone)]
    pub enum Qos {
        Low,   
        Medium, 
        High, 
    }

    pub enum Command {
        Publish { topic: String, message: String, qos: Qos },
        Subscribe { topic: String, qos: Qos },
    }

    #[derive(Clone)]
    pub struct Broker {
        pub tx: UnboundedSender<(Command, Option<UnboundedSender<(String, String)>>)>,
    }

    impl Broker {
        pub fn new() -> Self {
            let (tx, mut rx) = mpsc::unbounded_channel::<(Command, Option<UnboundedSender<(String, String)>>)>();

            tokio::spawn({
                async move {
                    let mut subscribers: HashMap<String, Vec<(UnboundedSender<(String, String)>, Qos)>> = HashMap::new();

                    while let Some((command, client_tx)) = rx.recv().await {
                        match command {

                            Command::Subscribe { topic, qos } => {
                                if let Some(subs) = subscribers.get_mut(&topic) {
                                    if let Some(tx) = client_tx {
                                        subs.push((tx, qos));
                                    }
                                } else if let Some(tx) = client_tx {
                                    subscribers.insert(topic, vec![(tx, qos)]);
                                }
                            }

                            Command::Publish { topic, message, qos } => {
                                if let Some(subs) = subscribers.get_mut(&topic) {
                                    for (sub, sub_qos) in subs.iter_mut() {
                                        if qos == Qos::Low || *sub_qos == Qos::Low {
                                            let _ = sub.send((topic.clone(), message.clone()));
                                            continue;
                                        } else if qos == Qos::High || *sub_qos == Qos::High {
                                            continue;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            });
            Broker { tx }
        }

        pub async fn start(&self, addr: &str) {
            let listener = TcpListener::bind(addr).await.expect("Failed to bind");
            let mut counter = 0;
            println!("[Broker]: waiting for connection");
            loop {
                match listener.accept().await {
                    Ok((stream, _)) => {
                        counter += 1;
                        println!("[Broker]: OK ---> Client {}", counter);
                        let tx = self.tx.clone();
                        tokio::spawn(async move {
                            handle_client(stream, tx).await;
                        });
                    }
                    Err(e) => eprintln!("Accept error: {}", e),
                }
            }
        }
    }

    async fn handle_client(
        stream: TcpStream,
        tx: UnboundedSender<(Command, Option<UnboundedSender<(String, String)>>)>,
    ) {
        let (mut reader, mut writer) = stream.into_split();
        let (client_tx, mut client_rx) = mpsc::unbounded_channel::<(String, String)>();

        //gửi dữ liệu đến các client subscribe
        tokio::spawn({
            async move {
                while let Some((topic, msg)) = client_rx.recv().await {
                    let formatted_message = format!("{} {}", topic, msg);
                    if writer.write_all(formatted_message.as_bytes()).await.is_err() {
                        eprintln!("Error sending message to client");
                        break;
                    }
                }
            }
        });


        let mut buffer = [0; 1024];
        loop {
            match reader.read(&mut buffer).await {
                Ok(n) if n == 0 => return,
                Ok(n) => {
                    let request = String::from_utf8_lossy(&buffer[..n]);

                    let parts: Vec<&str> = request.split_whitespace().collect();

                    if parts.len() < 3 {
                        eprintln!("Invalid request: {}", request);
                        continue;
                    }

                    match parts[0] {
                        "SUBSCRIBE" => {
                            let topic = parts[1].to_string();
                            let qos = parse_qos(parts[2]);
                            let _ = tx.send((Command::Subscribe { topic: topic.clone(), qos },Some(client_tx.clone())));
                            println!("SUBSCRIBE: [Client] ---> [Broker] - Topic: {}, QoS: {:?}", topic, qos);
                        }
                        "PUBLISH" => {
                            if parts.len() < 4 {
                                eprintln!("Invalid publish request: {}", request);
                                continue;
                            }
                            let topic = parts[1].to_string();
                            let message = parts[2].to_string();
                            let qos = parse_qos(parts[3]);
                            let _ = tx.send((Command::Publish { topic: topic.clone(), message: message.clone(), qos },None));
                            println!("PUBLISH  : [Client] ---> [Broker] - Topic: {}, Message: {}, QoS: {:?}",topic, message, qos);
                        }
                        _ => eprintln!("Unknown command: {}", parts[0]),
                    }
                }
                Err(e) => {
                    eprintln!("Error reading from client: {}", e);
                    return;
                }
            }
        }
    }

    fn parse_qos(qos: &str) -> Qos {
        match qos {
            "0" => Qos::Low,
            "1" => Qos::Medium,
            "2" => Qos::High,
            _   => Qos::Low,
        }
    }
}
