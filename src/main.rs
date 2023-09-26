use service::service::WebSocketClient;

mod service;
mod types;

#[tokio::main]
async fn main() {
    let client = WebSocketClient::new(String::from("172.172.194.77"), 8080);
    let registration_url = client.register(6).await;
    println!("{}", registration_url.url);
}
