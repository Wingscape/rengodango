use crate::DbCommand;
use rusqlite::Connection;
use tokio::sync::mpsc;

fn create_table(conn: &Connection) {
    let sql = "
    CREATE TABLE IF NOT EXISTS anime_user (
        id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
        user_id INTEGER,
        user_name TEXT,
        anime_id INTEGER,
        anime_title TEXT
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
            } => {
                store_anime_list(&conn, user_id, user_name, anime_id, anime_title);
            }
        }
    }
}
