use lib::mqtt::broker::broker::Broker;

#[tokio::main]
async fn main() {
    let broker = Broker::new();
    broker.start("127.0.0.1:8080").await;
}
