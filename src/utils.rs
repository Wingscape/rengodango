use crate::{AnimeInteraction, DbCommand};
use ab_glyph::{FontVec, PxScale};
use image::{Rgb, RgbImage};
use imageproc::drawing::draw_text;
use rand::seq::IndexedRandom;
use rusqlite::Connection;
use serenity::builder::{
    CreateActionRow, CreateAttachment, CreateButton, CreateEmbed, CreateEmbedFooter,
    CreateInteractionResponse, CreateInteractionResponseMessage,
};
use serenity::model::application::ButtonStyle;
use serenity::model::id::UserId;
use serenity::model::mention::Mention;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
struct AnimeList {
    anime_id: u64,
    anime_title: String,
}

fn create_table(conn: &Connection) {
    let sql = "
    CREATE TABLE IF NOT EXISTS anime_user (
        id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
        user_id INTEGER,
        user_name TEXT,
        anime_id INTEGER,
        anime_title TEXT,
        UNIQUE(user_id, anime_id) ON CONFLICT IGNORE
    ) STRICT";

    match conn.execute(sql, ()) {
        Ok(_) => {
            println!("Accessed a anime_user table.");
        }
        Err(e) => {
            panic!("Error connecting to table: {}", e)
        }
    }
}

fn store_anime_list(
    conn: &Connection,
    user_id: u64,
    user_name: String,
    anime_id: u64,
    anime_title: String,
) {
    let sql = "
    INSERT INTO anime_user (user_id, user_name, anime_id, anime_title)
    VALUES (?1, ?2, ?3, ?4)
    ";

    if let Ok(mut insert_sql) = conn.prepare(sql) {
        if let Err(why) = insert_sql.execute([
            user_id.to_string(),
            user_name,
            anime_id.to_string(),
            anime_title,
        ]) {
            println!("Database error: {why:?}");
        }
    }
}

fn is_anime_empty(conn: &Connection, user_id: u64, anime_id: u64) -> bool {
    let sql = "
    SELECT 1 FROM anime_user
    WHERE user_id = ?1 AND anime_id = ?2
    ";

    if let Ok(mut select_sql) = conn.prepare(sql) {
        if let Ok(mut data) = select_sql.query([user_id.to_string(), anime_id.to_string()]) {
            if let None = data.next().unwrap() {
                true
            } else {
                false
            }
        } else {
            panic!("Error at executing query");
        }
    } else {
        panic!("Error at preparing query");
    }
}

fn user_anime_list(conn: &Connection, user_id: u64, offset: u8, limit: u8) -> Vec<AnimeList> {
    let sql = "
    SELECT anime_id, anime_title FROM anime_user
    WHERE user_id = ?1 LIMIT ?3 OFFSET ?2
    ";

    let mut anime_list_final: Vec<AnimeList> = Vec::new();

    if let Ok(mut select_sql) = conn.prepare(sql) {
        let rows = match select_sql.query_map(
            [
                user_id.to_string(),
                offset.to_string(),
                (limit + 1).to_string(),
            ],
            |row| {
                Ok(AnimeList {
                    anime_id: row.get(0)?,
                    anime_title: row.get(1)?,
                })
            },
        ) {
            Ok(data) => data,
            Err(_) => panic!("Not good"),
        };

        for anime_list in rows {
            match anime_list {
                Ok(data) => anime_list_final.push(AnimeList {
                    anime_id: data.anime_id,
                    anime_title: data.anime_title,
                }),
                Err(_) => panic!("Error at executing query"),
            }
        }
    } else {
        panic!("Error at preparing query");
    }

    anime_list_final
}

fn draw_image(anime_list: Vec<AnimeList>, file_name: &str) {
    let mut white_bg = RgbImage::from_fn(400, 600, |_, _| Rgb([255, 255, 255]));

    let noto_bold_font = Vec::from(include_bytes!("../NotoSansHK-Bold.ttf") as &[u8]);
    let noto_bold_font = FontVec::try_from_vec(noto_bold_font).unwrap();

    let noto_reg_font = Vec::from(include_bytes!("../NotoSansHK-Regular.ttf") as &[u8]);
    let noto_reg_font = FontVec::try_from_vec(noto_reg_font).unwrap();

    // font size
    let scale = PxScale::from(30.0);

    let red = 50;
    let green = 50;
    let blue = 50;

    let mut y = 600;

    for anime in anime_list {
        let anime_title = anime.anime_title;

        let anime_title = match anime_title.char_indices().nth(32) {
            Some((idx, _)) => format!("{}...", anime_title[..idx].to_string()),
            None => anime_title,
        };

        white_bg = draw_text(
            &white_bg,
            Rgb([red, green, blue]),
            400 / 20,
            y / 20,
            scale,
            &noto_bold_font,
            anime_title.as_str(),
        );

        y = y + 600;

        white_bg = draw_text(
            &white_bg,
            Rgb([red, green, blue]),
            400 / 20,
            y / 20,
            scale,
            &noto_reg_font,
            format!("ID: {}", anime.anime_id).as_str(),
        );

        y = y + 1000;
    }

    white_bg.save(file_name).unwrap();
}

pub async fn open_connection(mut rx: mpsc::Receiver<DbCommand>) {
    let db_file = "./src/rengo.db";

    let conn = match Connection::open(db_file) {
        Ok(conn) => {
            println!("Database connection opened.");
            conn
        }
        Err(e) => {
            panic!("Error connecting to database: {}", e);
        }
    };

    while let Some(command) = rx.recv().await {
        println!("Processing: {:?}", command);

        match command {
            DbCommand::CreateTable => {
                create_table(&conn);
            }
            DbCommand::SaveAnime {
                user_id,
                user_name,
                anime_id,
                anime_title,
                component,
                ctx,
            } => {
                let message: String;
                let anime_empty = is_anime_empty(&conn, user_id, anime_id);
                let user_mention = UserId::new(user_id);

                if anime_empty {
                    store_anime_list(&conn, user_id, user_name, anime_id, anime_title);

                    let save_responses = vec![
                        "ehehe~ I saved your data safely ( ◡̀_◡́)ᕤ",
                        "yatta~! Your anime has been added successfully ◝(ᵔᵕᵔ)◜",
                        "nya~ I've got your anime right here! (⸝⸝ᵕᴗᵕ⸝⸝)",
                    ];

                    message = match save_responses.choose(&mut rand::rng()) {
                        Some(value) => {
                            format!("{} {}", Mention::from(user_mention), value)
                        }
                        None => format!("Oopsie! Something went wrong... (｡•́︿•̀｡)"),
                    };
                } else {
                    message = format!(
                        "{} this one's already saved, senpai~~ (¬_¬\")",
                        Mention::from(user_mention)
                    );
                }

                let data = CreateInteractionResponseMessage::new().content(message);
                let builder = CreateInteractionResponse::Message(data);

                if let Err(why) = component.create_response(&ctx.http, builder).await {
                    println!("cannot respond to slash command: {why}");
                }
            }
            DbCommand::ShowAnime {
                user_id,
                offset,
                limit,
                next,
                anime_interaction,
                ctx,
            } => {
                let user_mention = UserId::new(user_id);
                let file_name = "animelist.png";

                let anime_list = user_anime_list(&conn, user_id, offset, limit);
                let mut anime_draw = anime_list.clone();

                if anime_list.len() > 7 {
                    anime_draw = anime_draw[..7 as usize].to_vec();
                }

                draw_image(anime_draw, file_name);
                let path = CreateAttachment::path(file_name).await.unwrap();

                let list_embed = CreateEmbed::new()
                    .title(format!("Your precious Anime List~!"))
                    .description(format!("User: {}", Mention::from(user_mention)))
                    .footer(CreateEmbedFooter::new(format!(
                        "Page {}",
                        (next - 1).to_string()
                    )))
                    .image(format!("attachment://{}", file_name));

                let mut row_buttons = Vec::new();

                if offset != 0 {
                    let prev_button = CreateButton::new((next - 1).to_string())
                        .label("Prev")
                        .style(ButtonStyle::Secondary);

                    row_buttons.push(prev_button);
                }

                if anime_list.len() > 7 as usize {
                    let next_button = CreateButton::new(next.to_string())
                        .label("Next")
                        .style(ButtonStyle::Secondary);

                    row_buttons.push(next_button);
                }

                let row_components = CreateActionRow::Buttons(row_buttons);

                match anime_interaction {
                    // TODO: can we do something about this, like maybe trait implementation
                    AnimeInteraction::AnimeCommand(command) => {
                        let data = CreateInteractionResponseMessage::new()
                            .embed(list_embed)
                            .components(vec![row_components])
                            .add_file(path);

                        let builder = CreateInteractionResponse::Message(data);

                        if let Err(why) = command.create_response(&ctx.http, builder).await {
                            println!("cannot respond to slash command: {why}");
                        }
                    }
                    AnimeInteraction::AnimeComponent(component) => {
                        let data = CreateInteractionResponseMessage::new()
                            .embed(list_embed)
                            .components(vec![row_components])
                            .add_file(path);

                        let builder = CreateInteractionResponse::UpdateMessage(data);

                        if let Err(why) = component.create_response(&ctx.http, builder).await {
                            println!("cannot respond to slash command: {why}");
                        }
                    }
                };
            }
        }
    }
}
