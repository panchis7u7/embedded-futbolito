pub mod service {

    use std::error::Error;

    use futures_util::SinkExt;
    use http::HeaderValue;
    use log::debug;
    use log::info;
    use reqwest::header::ACCEPT;
    use reqwest::header::CONTENT_TYPE;

    // Tokio
    use tokio::sync::mpsc;
    use tokio::sync::mpsc::Receiver;
    use tokio::sync::mpsc::Sender;
    use tokio_tungstenite::WebSocketStream;
    use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

    // Future
    use futures_util::stream::{SplitSink, SplitStream};
    use futures_util::StreamExt;

    mod endpoints {
        // Private crate to hold all types that the user shouldn't have to interact with.
        use crate::types::RegisterResponse;
        use serde::Deserialize;
        // Trait for API types. Has to be public due to trait bounds limitations on webex API, but hidden
        // in a private crate so users don't see it.
        pub trait Gettable {
            const API_ENDPOINT: &'static str; // Endpoint to query to perform an HTTP GET request with or without an Id.
        }

        impl Gettable for RegisterResponse {
            const API_ENDPOINT: &'static str = "register";
        }

        #[derive(Deserialize)]
        pub struct ListResult<T> {
            pub items: Vec<T>,
        }
    }

    use crate::types::{Publish, Register, RegisterResponse};
    use http::HeaderMap;
    use reqwest::Client;

    use self::endpoints::Gettable;

    // Singleton class
    // ----------------------------------------------------------------------------
    pub struct WebSocketClient {
        host: String,
        port: u16,
        user_id: u16,
        subscription_groups: Vec<String>,
        _client: Client,
        _headers: HeaderMap,
    }

    impl WebSocketClient {
        pub fn new(
            host: &str,
            port: u16,
            user_id: u16,
            subscription_groups: Vec<String>,
        ) -> WebSocketClient {
            let mut headers = HeaderMap::new();
            headers.insert(
                CONTENT_TYPE,
                HeaderValue::from_str("application/json").unwrap(),
            );
            headers.insert(ACCEPT, HeaderValue::from_str("application/json").unwrap());

            WebSocketClient {
                host: String::from(host),
                port,
                user_id,
                subscription_groups,
                _client: Client::new(),
                _headers: headers,
            }
        }

        // WebSocket User Registration.
        // ----------------------------------------------------------------------------
        pub async fn register(&self) -> RegisterResponse {
            let response = self
                ._client
                .post(format!(
                    "http://{}:{}/{}",
                    self.host,
                    self.port,
                    RegisterResponse::API_ENDPOINT
                ))
                .headers(self._headers.clone())
                .json(&Register {
                    user_id: self.user_id,
                    groups: self.subscription_groups.clone(),
                })
                .send()
                .await
                .unwrap();

            self.review_status(&response);

            let message = response
                .json::<RegisterResponse>()
                .await
                .expect("failed to convert struct from json");

            return message;
        }

        // WebSocket message publishing.
        // ----------------------------------------------------------------------------

        pub async fn publish(&self, user_id: u16, group: String, message: serde_json::Value) {
            let response = self
                ._client
                .post(format!(
                    "http://{}:{}/{}",
                    self.host,
                    self.port,
                    RegisterResponse::API_ENDPOINT
                ))
                .headers(self._headers.clone())
                .json(&Publish {
                    user_id: user_id,
                    group,
                    message: message.to_string(),
                })
                .send()
                .await
                .unwrap();

            self.review_status(&response);
        }

        // Review the status for the response.
        // ----------------------------------------------------------------------------
        pub fn review_status(&self, response: &reqwest::Response) -> () {
            match response.status() {
                reqwest::StatusCode::OK => {
                    log::debug!("Succesful request: {:?}", response)
                }
                reqwest::StatusCode::NOT_FOUND => {
                    log::debug!("Got 404! Haven't found resource!: {:?}", response)
                }
                _ => {
                    log::error!("Got 404! Haven't found resource!: {:?}", response)
                }
            }
        }

        // Initialize WebSocket Client.
        // ----------------------------------------------------------------------------
        pub async fn start_ws_client(
            &self,
            registration_url: String,
        ) -> Result<(Sender<Message>, Receiver<Message>), Box<dyn Error>> {
            // Parse the registration URL as of a URL type.
            let url = url::Url::parse(&registration_url).unwrap();
            debug!("Parsed registration string: {}", url);

            // Create channels to send and receive messages
            let (sender, receiver) = mpsc::channel(32);

            // let (stdin_tx, stdin_rx) = futures_channel::mpsc::unbounded();
            // tokio::spawn(read_stdin(stdin_tx));

            let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
            info!("WebSocket handshake has been successfully completed");

            // Split the WebSocket into sender and receiver.
            let (ws_sender, ws_receiver) = ws_stream.split();

            // Spawn a task to receive messages and forward them to the receiver channel
            tokio::spawn(receive_messages(ws_receiver, sender.clone()));

            // Spawn a task to send messages
            tokio::spawn(send_messages(ws_sender));

            Ok((sender, receiver))
        }
    }

    // Function to receive messages from the WebSocket and forward them to the channel.
    // ----------------------------------------------------------------------------
    async fn receive_messages(
        ws_stream: SplitStream<
            WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
        >,
        sender: Sender<Message>,
    ) {
        let mut ws_stream = ws_stream;

        while let Some(message) = ws_stream.next().await {
            match message {
                Ok(msg) => {
                    // Forward the received message to the channel
                    if sender.send(msg).await.is_err() {
                        eprintln!("Receiver dropped, closing connection.");
                        return;
                    }
                }
                Err(e) => {
                    eprintln!("Error receiving message: {}", e);
                }
            }
        }
    }

    // Function to send messages via the WebSocket.
    // ----------------------------------------------------------------------------
    pub async fn send_messages(
        mut ws_stream: SplitSink<
            WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
            tokio_tungstenite::tungstenite::Message,
        >,
    ) {
        // This could be a loop where you send messages as needed
        // For the example, we're just sending one message and then exiting
        let message = Message::Text("Hello, WebSocket Server!".into());

        if let Err(e) = ws_stream.send(message).await {
            eprintln!("Error sending message: {}", e);
        }
    }
}
