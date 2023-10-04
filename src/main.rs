// #########################################################################################
// Imports
// #########################################################################################

// std
use std::io::{self, Write};
use std::thread;

// futures
use futures_util::{SinkExt, StreamExt};

// log
use log::{debug, info};

// local
use parser::{Argument, Parser};
use rusty_webex::WebexClient;
use service::service::WebSocketClient;
use types::{ArgTuple, MessageEventResponse, Response};

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

mod parser;
mod service;
mod types;

// #########################################################################################
// Callbacks
// #########################################################################################

pub fn say_hello<'a>(required_args: &'a ArgTuple, optional_args: &'a ArgTuple) -> () {
    debug!(
        "Required arguments in the callback: {}\n",
        required_args.len()
    );
    debug!(
        "Optional arguments in the callback: {}\n",
        optional_args.len()
    );
}

// #########################################################################################
// Webhook root listener.
// #########################################################################################

#[post("/cats/futbolito", format = "json", data = "<data>")]
async fn webhook_listener<'a>(data: Json<Response<MessageEventResponse>>) -> () {
    info!("[Webhook data]: {:?}\n", data);

    // Load the environment variables from the .env file.
    if dotenv().ok().is_none() {
        some_error(".env file not detected.");
    }

    // Create a new webex client.
    let client = WebexClient::new(
        std::env::var("TOKEN")
            .expect("The TOKEN must be set.")
            .as_str(),
    );

    // Retrieve message details as this contains the text for the bot call.
    let detailed_message_info = client.get_message_details(&data.data.id).await;

    // Log the detailed message contents.
    log::info!("[Message info]: {:?}\n", &detailed_message_info);

    // Create a new parser that can interpret the available commands.
    let mut parser = Parser::new();
    parser.add_command("/embedded", vec![], say_hello);
    parser.add_command("/casual_tournament", vec![], say_hello);
    parser.add_command("/pair_tournament", vec![], say_hello);
    parser.add_command("/say_hello", vec![], say_hello);

    // Parse the received message.
    parser.parse(detailed_message_info.text.unwrap());
}

#[post("/signature")]
fn signature() -> &'static str {
    "embedded fudbolito bot!"
}

// Error utility function.
fn some_error(msg: &str) -> ! {
    eprintln!("Error: {}", msg);
    panic!();
}

// #########################################################################################
// Intialize Websocket Client for the Embedded Communication.
// #########################################################################################

async fn init_websocket_client() {
    // Register to the websocket server..
    let client = WebSocketClient::new(String::from("172.172.194.77"), 8080);
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

    let _rocket = rocket::build()
        .mount("/", routes![signature, webhook_listener])
        .mount("/public", FileServer::from("static/"))
        .launch()
        .await?;
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
