mod commands;

use serenity::all::{CreateInteractionResponse, CreateInteractionResponseMessage, GatewayIntents};
use serenity::async_trait;
use serenity::model::application::{Command, Interaction};
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::prelude::{Client, Context, EventHandler};
use std::env;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    // Dispatched when an interaction is created (e.g a slash command was used or a button was clicked).
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            println!("received command interaction: {command:?}");

            let content = match command.data.name.as_str() {
                "anime" => Some(commands::anime::run(&command.data.options()).await),
                _ => Some("this feature is not implemented :(".to_string()),
            };

            if let Some(content) = content {
                let data = CreateInteractionResponseMessage::new().content(content);
                let builder = CreateInteractionResponse::Message(data);

                if let Err(why) = command.create_response(&ctx.http, builder).await {
                    println!("cannot respond to slash command: {why}");
                }
            }
        }
    }

    // Dispatched upon startup
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let guild_id = GuildId::new(
            env::var("GUILD_ID")
                .expect("expected GUILD_ID in environment")
                .parse()
                .expect("GUILD_ID must be an integer"),
        );

        let commands = guild_id
            .set_commands(&ctx.http, vec![commands::anime::register()])
            .await;

        match commands {
            Ok(bla) => println!("guild slash command: {bla:?}"),
            Err(e) => println!("error: {e}"),
        }

        // TODO: create a global command
        // let commands = Command::create_global_command(&ctx.http, commands::hello::register()).await;

        // println!("guild slash command: {commands:#?}");
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("weird");
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
