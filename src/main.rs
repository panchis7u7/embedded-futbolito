// #########################################################################################
// Imports
// #########################################################################################

// std
use std::io::{self, Write};
use std::{thread, vec};

// futures
use futures_util::{Future, SinkExt, StreamExt};

// log
use log::{debug, info};

// local
use rusty_webex::types::MessageOut; //For pull from git.
use rusty_webex::WebexBotServer;
use rusty_webex::WebexClient;
use service::service::WebSocketClient;
use types::{MessageEventResponse, Response};

// dotenv
use dotenv::dotenv;

// Tokio
use tokio::io::AsyncReadExt;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

// Rocket Dependencies
use rocket::fs::FileServer;
use rocket::serde::json::Json;
use rocket::{post, routes};

// #########################################################################################
// Modules
// #########################################################################################

extern crate rusty_webex;
mod service;
mod types;

pub type ArgTuple = Vec<(std::string::String, std::string::String)>;

// #########################################################################################
// Utility functions.
// #########################################################################################

fn some_error(msg: &str) -> ! {
    eprintln!("Error: {}", msg);
    panic!();
}

// #########################################################################################
// Intialize Websocket Client for the Embedded Communication.
// #########################################################################################

async fn init_websocket_client() {
    // Register to the websocket server..
    let client = WebSocketClient::new(
        String::from(
            std::env::var("WS_SERVER")
                .expect("The TOKEN must be set.")
                .as_str(),
        ),
        std::env::var("WS_SERVER_PORT")
            .expect("The TOKEN must be set.")
            .parse::<u16>()
            .unwrap(),
    );
    let registration_url = client.register(6).await;
    println!("Registration URL from server: {}", &registration_url.url);

    // Parse the registration URL as of a URL type.
    let url = url::Url::parse(&registration_url.url).unwrap();
    debug!("Parsed registration string: {}", url);

    // Retrieve the ws stream.
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    info!("WebSocket handshake has been successfully completed");

    // Split the full-duplex stream into sender and receiver.
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
}

// #########################################################################################
// Server Entrypoint.
// #########################################################################################

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    ::std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    // Load the environment variables from the .env file.
    if dotenv().ok().is_none() {
        some_error(".env file not detected.");
    }

    // Create a new webex bot server.
    let server = WebexBotServer::new(
        std::env::var("TOKEN")
            .expect("The TOKEN must be set.")
            .as_str(),
    );

    // server.add_command(
    //     "/say_hello",
    //     vec![],
    //     Box::new(
    //         move |client: &WebexClient, message, required_arguments, optional_arguments| {
    //             Box::pin(async move {
    //                 log::info!("Callback executed!");
    //                 client.send_message(&MessageOut::from(message)).await;
    //             })
    //         },
    //     ),
    // );

    server
        .add_command(
            "/say_hello",
            vec![],
            move |client: WebexClient, message, _required_args, _optional_argss| {
                Box::pin(async move {
                    log::info!("Callback executed!");

                    let mut event_response_message = MessageOut::from(message);
                    event_response_message.text =
                        Some("Hola desde el cliente de Rust!".to_string());
                    client
                        .send_message(&MessageOut::from(event_response_message))
                        .await;
                })
            },
        )
        .await;

    // Launch the server.
    server.launch().await;

    Ok(())
}

/*
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ::std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let client = WebSocketClient::new(String::from("172.172.194.77"), 8080);
    let registration_url = client.register(6).await;
    println!("Registration URL from server: {}", &registration_url.url);

    // Parse the registration URL as of a URL type.
    let url = url::Url::parse(&registration_url.url).unwrap();
    debug!("Parsed registration string: {}", url);

    let (stdin_tx, stdin_rx) = futures_channel::mpsc::unbounded();
    tokio::spawn(read_stdin(stdin_tx));

    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    info!("WebSocket handshake has been successfully completed");

    // Split the WebSocket into sender and receiver.
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    // let stdin_to_ws = stdin_rx.map(Ok).forward(ws_sender);
    // let ws_to_stdout = {
    //     ws_receiver.for_each(|message| async {
    //         let data = message.unwrap().into_data();
    //         tokio::io::stdout().write_all(&data).await.unwrap();
    //     })
    // };

    // -----------------------------------------------------------------
    // stdin/stdout is used for testing purposes.
    // -----------------------------------------------------------------
    // Create a separate thread for sending messages.
    let send_thread = thread::spawn(move || {
        let stdin = io::stdin();
        let mut stdout = io::stdout();
        let mut input = String::new();

        loop {
            input.clear();
            print!("Enter a message: ");
            stdout.flush().unwrap();
            stdin.read_line(&mut input).unwrap();

            let trimmed = input.trim();
            if trimmed.is_empty() {
                continue;
            }

            let message = Message::Text(trimmed.to_string());

            ws_sender.send(message);
        }
    });

    // Handle incoming WebSocket messages
    while let Some(Ok(msg)) = ws_receiver.next().await {
        match msg {
            Message::Text(text) => {
                println!("Received text message: {}", text);
            }
            Message::Binary(bin_data) => {
                println!("Received binary message with {} bytes", bin_data.len());
            }
            Message::Close(_) => {
                println!("WebSocket connection closed");
                break;
            }
            _ => {
                // Handle other message types as needed
            }
        }
    }

    // Wait for the send thread to finish
    send_thread.join().unwrap();

    // pin_mut!(stdin_to_ws, ws_to_stdout);
    // future::select(stdin_to_ws, ws_to_stdout).await;

    Ok(())
}*/

// Our helper method which will read data from stdin and send it along the
// sender provided.
async fn read_stdin(tx: futures_channel::mpsc::UnboundedSender<Message>) {
    let mut stdin = tokio::io::stdin();
    loop {
        let mut buf = vec![0; 1024];
        let n = match stdin.read(&mut buf).await {
            Err(_) | Ok(0) => break,
            Ok(n) => n,
        };
        buf.truncate(n);
        tx.unbounded_send(Message::binary(buf)).unwrap();
    }
}
