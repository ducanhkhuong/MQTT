use lib::mqtt::client::client::Client;

fn main() {
    match Client::new("127.0.0.1:1883") {
        Ok(mut client) => {
            client.start();
        }
        Err(e) => {
            eprintln!("Failed to connect to broker: {}", e);
        }
    }
}
