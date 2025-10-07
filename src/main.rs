mod commands;

use serenity::all::{CreateInteractionResponse, CreateInteractionResponseMessage, GatewayIntents};
use serenity::async_trait;
use serenity::model::application::{Command, Interaction};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::prelude::{Client, Context, EventHandler};
use std::env;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    // // Dispatched when a message is created.
    // async fn message(&self, ctx: Context, msg: Message) {
    //     if msg.content == "!ping" {
    //         if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
    //             println!("Error sending message: {why:?}");
    //         }
    //     }

    //     if msg.content == "hello" {
    //         if let Err(why) = msg.channel_id.say(&ctx.http, "Holla").await {
    //             println!("Error sending message: {why:?}");
    //         }
    //     }
    // }

    // Dispatched when an interaction is created (e.g a slash command was used or a button was clicked).
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            println!("{command:#?}");

            let content = match command.data.name.as_str() {
                "hello" => Some(commands::hello::run(&command.data.options())),
                _ => Some("not implemented :".to_string()),
            };

            if let Some(content) = content {
                let data = CreateInteractionResponseMessage::new().content(content);
                let builder = CreateInteractionResponse::Message(data);

                if let Err(why) = command.create_response(&ctx.http, builder).await {
                    println!("Cannot respond to slash command: {why}");
                }
            }
        }
    }

    // Dispatched upon startup
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let guild_id = GuildId::new(
            env::var("GUILD_ID")
                .expect("Expected GUILD_ID in environment")
                .parse()
                .expect("GUILD_ID must be an integer"),
        );

        let commands = guild_id
            .set_commands(&ctx.http, vec![commands::hello::register()])
            .await;

        println!("guild slash command: {commands:#?}");
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("weird");
    // let intents = GatewayIntents::GUILD_MESSAGES
    //     | GatewayIntents::DIRECT_MESSAGES
    //     | GatewayIntents::MESSAGE_CONTENT;

    let intents = GatewayIntents::empty();

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    // TODO: consider transparent sharding when the app getting too big
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
