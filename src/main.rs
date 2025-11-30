mod commands;
mod interactions;
mod utils;
use serenity::async_trait;
use serenity::model::application::{CommandInteraction, ComponentInteraction, Interaction};
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::prelude::{Client, Context, EventHandler, GatewayIntents};
use std::env;
use tokio::sync::mpsc;

#[derive(Debug)]
enum DbCommand {
    CreateTable,
    SaveAnime {
        user_id: u64,
        user_name: String,
        anime_id: u64,
        anime_title: String,
        component: ComponentInteraction,
        ctx: Context,
    },
    ShowAnime {
        user_id: u64,
        command: CommandInteraction,
        ctx: Context,
    },
}

struct Handler {
    tx: mpsc::Sender<DbCommand>,
}

#[async_trait]
impl EventHandler for Handler {
    // Dispatched when an interaction is created (e.g a slash command was used or a button was clicked).
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        // Debugging purpose
        // println!("{:#?}, {:#?}", ctx, interaction);

        if let Interaction::Component(component) = &interaction {
            let tx_save = self.tx.clone();
            interactions::component::create_anime_embed(&ctx, component).await;
            interactions::component::store_anime_list(&ctx, component, tx_save).await;
        }

        if let Interaction::Command(command) = &interaction {
            let tx_display = self.tx.clone();
            interactions::command::create_anime_select(&ctx, command).await;
            interactions::command::display_anime_list(&ctx, command, tx_display).await;
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
            .set_commands(
                &ctx.http,
                vec![commands::anime::register(), commands::show::register()],
            )
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
    let (tx, rx) = mpsc::channel(100);
    let tx_2 = tx.clone();

    tokio::spawn(utils::open_connection(rx));

    tokio::spawn(async move {
        tx.send(DbCommand::CreateTable).await.unwrap();
    });

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::empty();

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler { tx: tx_2 })
        .await
        .expect("Error creating client");

    // TODO: consider transparent sharding when the app getting too big
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
