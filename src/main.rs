mod flikisdining;
mod lunch;

use serenity::{prelude::GatewayIntents, Client};

use std::env;

#[tokio::main]
async fn main() {
    // load env
    let _ = dotenvy::dotenv();

    // get the token
    let token = env::var("TOKEN").expect("Expected a token in the environment");

    // set the intents
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    // create the client
    let mut client = Client::builder(&token, intents)
        .event_handler(lunch::Handler)
        .await
        .expect("Failed to create client");

    // start the client
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
