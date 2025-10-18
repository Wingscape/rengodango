mod commands;

use serenity::async_trait;
use serenity::builder::{
    CreateButton, CreateEmbed, CreateInteractionResponse, CreateInteractionResponseMessage,
    CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption,
};
use serenity::model::application::{ButtonStyle, ComponentInteractionDataKind, Interaction};
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::prelude::{Client, Context, EventHandler, GatewayIntents};
use std::env;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    // Dispatched when an interaction is created (e.g a slash command was used or a button was clicked).
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        println!("{:#?}, {:#?}", ctx, interaction);

        if let Interaction::Component(component) = &interaction {
            let content = match component.data.custom_id.as_str() {
                "anime_select" => match &component.data.kind {
                    ComponentInteractionDataKind::StringSelect { values } => {
                        if let Some(value_select) = values.first() {
                            Some(commands::anime_id::run(value_select.as_str()).await)
                        } else {
                            None
                        }
                    }
                    _ => None,
                },
                _ => None,
            };

            if let Some(content) = content {
                let anime_embed = CreateEmbed::new()
                    .title(format!("{}", content.title))
                    .url(format!("{}", content.url))
                    .image(format!("{}", content.image))
                    .description(format!(
                        "ID: {}, Synopsis: {}",
                        content.id, content.synopsis
                    ));

                let anime_button = CreateButton::new_link(format!(
                    "https://myanimelist.net/anime/21273/Gochuumon_wa_Usagi_desu_ka"
                ))
                .label("Add")
                .style(ButtonStyle::Primary);

                let data = CreateInteractionResponseMessage::new()
                    .embed(anime_embed)
                    .button(anime_button);
                let builder = CreateInteractionResponse::Message(data);

                if let Err(why) = component.create_response(&ctx.http, builder).await {
                    println!("cannot respond to slash command: {why}");
                }
            }
        }

        if let Interaction::Command(command) = &interaction {
            let content = match command.data.name.as_str() {
                "anime" => Some(commands::anime::run(&command.data.options()).await),
                _ => None,
            };

            if let Some(content) = content {
                let anime_menu_options: Vec<CreateSelectMenuOption> = content
                    .iter()
                    .take(7)
                    .map(|x| CreateSelectMenuOption::new(x.title.clone(), x.id.to_string().clone()))
                    .collect();

                let anime_select = CreateSelectMenu::new(
                    "anime_select",
                    CreateSelectMenuKind::String {
                        options: anime_menu_options,
                    },
                )
                .placeholder("Which one? >_<")
                .min_values(0)
                .max_values(1);

                let data = CreateInteractionResponseMessage::new().select_menu(anime_select);
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
