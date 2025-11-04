use crate::DbCommand;
use rand::seq::IndexedRandom;
use rusqlite::Connection;
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::model::id::UserId;
use serenity::model::mention::Mention;
use tokio::sync::mpsc;

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
        }
    }
}
