use crate::commands;
use serenity::builder::{
    CreateInteractionResponse, CreateInteractionResponseMessage, CreateSelectMenu,
    CreateSelectMenuKind, CreateSelectMenuOption,
};
use serenity::model::application::CommandInteraction;
use serenity::prelude::Context;

pub async fn create_anime_select(ctx: &Context, command: &CommandInteraction) {
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
