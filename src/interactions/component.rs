use crate::interactions::anime_id;
use crate::{AnimeInteraction, DbCommand};
use serenity::builder::{
    CreateButton, CreateEmbed, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use serenity::model::application::{
    ActionRowComponent, ButtonStyle, ComponentInteraction, ComponentInteractionDataKind,
};
use serenity::prelude::Context;
use tokio::sync::mpsc;

pub async fn create_anime_embed(ctx: &Context, component: &ComponentInteraction) {
    let content = match component.data.custom_id.as_str() {
        "anime_select" => match &component.data.kind {
            ComponentInteractionDataKind::StringSelect { values } => {
                if let Some(value_select) = values.first() {
                    Some(anime_id::run(value_select.as_str()).await)
                } else {
                    None
                }
            }
            _ => None,
        },
        _ => None,
    };

    if let Some(content) = content {
        let synopsis = content.synopsis;

        let synopsis = match synopsis.char_indices().nth(1020) {
            Some((idx, _)) => synopsis[..idx].to_string(),
            None => synopsis,
        };

        let fields = [
            ("ID", content.id.to_string(), false),
            ("Synopsis", synopsis.to_string(), false),
        ];

        let anime_embed = CreateEmbed::new()
            .title(format!("{}", content.title))
            .url(format!("{}", content.url))
            .image(format!("{}", content.image))
            .fields(fields);

        let anime_button = CreateButton::new("anime_button")
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

pub async fn store_anime_list(
    ctx: &Context,
    component: &ComponentInteraction,
    tx_save: mpsc::Sender<DbCommand>,
) {
    match component.data.custom_id.as_str() {
        "anime_button" => match &component.data.kind {
            ComponentInteractionDataKind::Button => {
                let user_id = component.user.id.get();
                let user_name = component.user.name.to_string();
                let anime_id: u64 = component.message.embeds[0].fields[0]
                    .value
                    .clone()
                    .parse()
                    .expect("Failed to parse string to integer");

                let anime_title = match &component.message.embeds[0].title {
                    Some(value) => value.clone(),
                    None => "no".to_string(),
                };

                let component_save = component.clone();
                let ctx_save = ctx.clone();

                tokio::spawn(async move {
                    tx_save
                        .send(DbCommand::SaveAnime {
                            user_id: user_id,
                            user_name: user_name,
                            anime_id: anime_id,
                            anime_title: anime_title,
                            component: component_save,
                            ctx: ctx_save,
                        })
                        .await
                        .unwrap();
                });
            }
            _ => println!("wow"),
        },
        _ => println!("wow"),
    };
}

pub async fn display_anime_list_page(
    ctx: &Context,
    component: &ComponentInteraction,
    tx_display: mpsc::Sender<DbCommand>,
) {
    let component_show = component.clone();
    let ctx_show = ctx.clone();

    let mut custom_id: u8 = component.data.custom_id.to_string().parse().unwrap();
    let user_id = component.user.id.get();

    let base_number = 7;
    let mut page: (u8, u8) = (base_number * (custom_id - 1), base_number * custom_id);
    let action_component = &component.message.components[0].components[0];

    match action_component {
        ActionRowComponent::Button(button_action) => match &button_action.label {
            Some(label) => {
                if label == "Prev" {
                    page = (base_number * (custom_id - 2), base_number * (custom_id - 1));
                    custom_id = custom_id - 1;
                }
            }
            None => println!("wow"),
        },
        _ => println!("wow"),
    }

    tokio::spawn(async move {
        tx_display
            .send(DbCommand::ShowAnime {
                user_id: user_id,
                offset: page.0,
                limit: page.1,
                next: custom_id + 1,
                anime_interaction: AnimeInteraction::AnimeComponent(component_show),
                ctx: ctx_show,
            })
            .await
            .unwrap();
    });
}
