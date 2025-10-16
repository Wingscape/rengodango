mod commands;

use serenity::all::{CreateButton, CreateSelectMenu, CreateSelectMenuKind};
use serenity::async_trait;
use serenity::builder::{
    CreateEmbed, CreateInteractionResponse, CreateInteractionResponseMessage,
    CreateSelectMenuOption,
};
use serenity::model::application::Interaction;
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::prelude::{Client, Context, EventHandler, GatewayIntents};
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
                _ => None,
            };

            if let Some(content) = content {
                // TODO: do something about multiple data vector from content
                let anime_button = CreateButton::new_link(format!(
                    "https://myanimelist.net/anime/21273/Gochuumon_wa_Usagi_desu_ka"
                ))
                .label("Save?");

                let anime_embeds: Vec<CreateEmbed> = content
                    .iter()
                    .take(3)
                    .map(|x| {
                        CreateEmbed::new()
                            .title(format!("{}", x.title))
                            .url(format!("{}", x.url))
                            .image(format!("{}", x.image))
                            .description(format!("ID: {}, Synopsis: {}", x.id, x.synopsis))
                    })
                    .collect();

                let anime_menu_options: Vec<CreateSelectMenuOption> = content
                    .iter()
                    .take(3)
                    .map(|x| CreateSelectMenuOption::new(x.title.clone(), x.title.clone()))
                    .collect();

                let anime_select = CreateSelectMenu::new(
                    "anime_select",
                    CreateSelectMenuKind::String {
                        options: anime_menu_options,
                    },
                );

                let data = CreateInteractionResponseMessage::new()
                    .embeds(anime_embeds)
                    .select_menu(anime_select)
                    .button(anime_button);
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
