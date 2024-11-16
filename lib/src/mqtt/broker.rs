pub mod broker{
    use tokio::sync::mpsc;
    use tokio::net::{TcpListener, TcpStream};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use std::collections::HashMap;

    #[derive(Clone)]
    pub struct Broker {
        pub tx: mpsc::UnboundedSender<(String, String, Option<mpsc::UnboundedSender<String>>)>,
    }

    impl Broker {
        pub fn new() -> Self {
            let (tx, mut rx) = mpsc::unbounded_channel::<(String, String, Option<mpsc::UnboundedSender<String>>)>();
            
            tokio::spawn({
                async move {
                    let mut subscribers: HashMap<String, Vec<mpsc::UnboundedSender<String>>> = HashMap::new();
                    while let Some((topic, message, client_tx)) = rx.recv().await {
                        if message.is_empty() {
                            // Handle subscription
                            if let Some(subs) = subscribers.get_mut(&topic) {
                                if let Some(tx) = client_tx {
                                    subs.push(tx);
                                }
                            } else {
                                if let Some(tx) = client_tx {
                                    subscribers.insert(topic.clone(), vec![tx]);
                                }
                            }
                        } else {
                            // Handle publishing
                            if let Some(subs) = subscribers.get_mut(&topic) {
                                for sub in subs.iter_mut() {
                                    let _ = sub.send(message.clone());
                                }
                            }
                        }
                    }
                }
            });
            Broker { tx }
        }

        pub async fn start(&self, addr: &str) {
            let listener = TcpListener::bind(addr)
                .await
                .expect("Failed to bind to address");
            let mut counter  = 0;
            loop {
                match listener.accept().await {
                    Ok((stream, _)) => {
                        counter+=1;
                        println!("[Broker]  : OK --- Client : {:?}",counter);
                        let tx = self.tx.clone();
                        tokio::spawn({
                            async move {
                                handle_client(stream, tx).await;
                            }
                        });
                    }
                    Err(e) => {
                        eprintln!("Error accepting connection: {}", e);
                    }
                }
            }
        }
    }

    async fn handle_client(stream: TcpStream,tx: mpsc::UnboundedSender<(String, String, Option<mpsc::UnboundedSender<String>>)>,) {
        let (mut reader, mut writer) = stream.into_split();
        let (client_tx, mut client_rx) = mpsc::unbounded_channel::<String>();

        tokio::spawn({
            async move {
                while let Some(msg) = client_rx.recv().await {
                    if writer.write_all(msg.as_bytes()).await.is_err() {
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

                    let parts: Vec<&str> = request.splitn(3, ' ').collect();

                    if parts.len() < 2 {
                        eprintln!("Invalid request: {}", request);
                        continue;
                    }

                    match parts[0] {
                        "SUBSCRIBE" => {
                            let topic = parts[1].to_string();
                            let _ = tx.send((topic.clone(), String::new(), Some(client_tx.clone())));
                            println!("SUBSCRIBE: [Client] ---> [Broker] - Topic: {}", topic);
                        }
                        "PUBLISH" => {
                            if parts.len() < 3 {
                                eprintln!("Invalid publish request: {}", request);
                                continue;
                            }
                            let topic = parts[1].to_string();
                            let message = parts[2].to_string();
                            let _ = tx.send((topic.clone(), message.clone(), None));
                            println!("PUBLISH: [Client] ---> [Broker] - Topic: {} , Message: {}", topic, message);
                        }
                        _ => {
                            eprintln!("Unknown command: {}", parts[0]);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error reading from client: {}", e);
                    return;
                }
            }
        }
    }
}
