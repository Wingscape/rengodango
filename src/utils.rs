use crate::DbCommand;
use ab_glyph::{FontVec, PxScale};
use image::{Rgb, RgbImage};
use imageproc::drawing::draw_text;
use rand::seq::IndexedRandom;
use rusqlite::Connection;
use serenity::builder::{
    CreateAttachment, CreateEmbed, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use serenity::model::id::UserId;
use serenity::model::mention::Mention;
use tokio::sync::mpsc;

#[derive(Debug)]
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

fn user_anime_list(conn: &Connection, user_id: u64) -> Vec<AnimeList> {
    let sql = "
    SELECT anime_id, anime_title FROM anime_user
    WHERE user_id = ?1
    ";

    let mut anime_list_final: Vec<AnimeList> = Vec::new();

    if let Ok(mut select_sql) = conn.prepare(sql) {
        let rows = match select_sql.query_map([user_id.to_string()], |row| {
            Ok(AnimeList {
                anime_id: row.get(0)?,
                anime_title: row.get(1)?,
            })
        }) {
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
                command,
                ctx,
            } => {
                let anime_list = user_anime_list(&conn, user_id);
                let mut white_bg = RgbImage::from_fn(400, 600, |_, _| Rgb([255, 255, 255]));

                let font = Vec::from(include_bytes!("/Library/Fonts/Arial Unicode.ttf") as &[u8]);
                let font = FontVec::try_from_vec(font).unwrap();

                // font size
                let scale = PxScale::from(30.0);

                let red = 50;
                let green = 50;
                let blue = 50;

                let mut y = 600;

                for anime in anime_list {
                    white_bg = draw_text(
                        &white_bg,
                        Rgb([red, green, blue]),
                        400 / 20,
                        y / 20,
                        scale,
                        &font,
                        anime.anime_title.as_str(),
                    );

                    y = y + 900;
                }

                white_bg.save("animelist.png").unwrap();

                let path = CreateAttachment::path("animelist.png").await.unwrap();

                let list_embed = CreateEmbed::new()
                    .title(format!("test"))
                    .image("attachment://animelist.png");

                let data = CreateInteractionResponseMessage::new()
                    .embed(list_embed)
                    .add_file(path);
                let builder = CreateInteractionResponse::Message(data);

                if let Err(why) = command.create_response(&ctx.http, builder).await {
                    println!("cannot respond to slash command: {why}");
                }
            }
        }
    }
}
