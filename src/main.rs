// #########################################################################################
// Imports
// #########################################################################################

// std
use std::vec;

// log
use log::debug;

use rusty_webex::adaptive_card::AdaptiveCard;
use rusty_webex::types::Attachment;
use rusty_webex::WebSocketClient;
// local
use rusty_webex::types::MessageOut; //For pull from git.
use rusty_webex::types::RequiredArgument;
use rusty_webex::WebexBotServer;

// dotenv
use dotenv::dotenv;

// Tokio
use tokio::sync::mpsc::Receiver;
use tokio_tungstenite::tungstenite::protocol::Message;

// Rocket Dependencies

// #########################################################################################
// Modules
// #########################################################################################

extern crate rusty_webex;
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
// Utility functions.
// #########################################################################################

// #########################################################################################
// Server Entrypoint.
// #########################################################################################

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    // -------------------------------------------------------------------------------------------
    // Say Hello (Greet) Command.
    // -------------------------------------------------------------------------------------------

    server
        .add_command(
            "/say_hello",
            vec![],
            move |client, message, _required_args, _optional_args| {
                Box::pin(async move {
                    debug!("[bot_server:say_hello]: callback entered.");

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

    // -------------------------------------------------------------------------------------------
    // Enable embedded functionality.
    // -------------------------------------------------------------------------------------------

    server
        .add_command(
            "/embedded",
            vec![Box::new(RequiredArgument::<String>::new("is_embedded"))],
            move |_client, _message, _required_args, _optional_args| {
                Box::pin(async move {
                    debug!("Activated embedded version for the futbolito bot!");

                    // Setup the websocket client for communication with the embedded device to the webex bot.
                    let ws_client = WebSocketClient::new(
                        "172.172.194.77",
                        8080,
                        2,
                        vec![String::from("fut_assist")],
                    );
                    let registration_url = ws_client.register("register").await;
                    println!("Registration URL from server: {}", &registration_url.url);

                    // Generate sender and receiver for the websocket crated.
                    let (_sender, receiver) = ws_client
                        .start_ws_client(registration_url.url)
                        .await
                        .unwrap();

                    // Spawn a task to listen for incoming messages
                    tokio::spawn(listen_for_messages(ws_client, receiver));
                })
            },
        )
        .await;

    // -------------------------------------------------------------------------------------------
    // Start a casual tournament.
    // -------------------------------------------------------------------------------------------

    server
        .add_command(
            "/casual_tournament",
            vec![],
            move |client, message, _r_args, _o_args| {
                Box::pin(async move {
                    debug!("[bot_server:casual_tournament]: callback entered.");

                    // Retrieve information from the bot reference message and the originator to
                    // save them later in a state.
                    let _message_id = message.id.clone().unwrap();
                    let _organizer_id = message.person_email.clone().unwrap();

                    // Generate an outoging message template from the incoming messages and edit only
                    // the fields of interest.
                    let mut event_response_message = MessageOut::from(message);

                    // Send the loaded adaptive card.
                    event_response_message.attachments = Some(vec![Attachment {
                        content_type: "application/vnd.microsoft.card.adaptive".to_string(),
                        content: AdaptiveCard::from_json_file_reader(
                            "./templates/card_template.json",
                        ),
                    }]);

                    // Send the card via the webex bot.
                    client.send_message(&event_response_message).await;

                    debug!("[bot_server:casual_tournament]: exiting callback.");
                })
            },
        )
        .await;

    // Launch the server.
    let _ = server.launch().await;

    Ok(())
}

// Function to listen for incoming messages and process them
async fn listen_for_messages(client: WebSocketClient, mut receiver: Receiver<Message>) {
    while let Some(message) = receiver.recv().await {
        match message {
            Message::Text(text) => {
                println!("Received message: {}", text);
                client
                    .publish(
                        "publish",
                        2,
                        String::from("embedded_rpi"),
                        serde_json::Value::String(String::from("{'message': 'Hello RPI!'}")),
                    )
                    .await;
            }
            _ => {
                println!("Received non-text message");
            }
        }
    }
}
