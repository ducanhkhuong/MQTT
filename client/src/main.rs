use lib::mqtt::client::client::Client;
use std::io::{self,BufRead};

fn main() {
    let mut client = Client::new("127.0.0.1:8080").expect("Failed to create MQTT client");
    println!("1 : Subscribe");
    println!("2 : Publish");
    loop {
        let stdin = io::stdin();
        let mut input = String::new();
        stdin.lock().read_line(&mut input).expect("Failed to read line");
        let choice = input.trim();

        client.start(choice);
    }
}
