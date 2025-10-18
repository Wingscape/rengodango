use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::model::application::{CommandOptionType, ResolvedOption, ResolvedValue};

pub struct MALAnime {
    pub id: i64,
    pub title: String,
}

pub async fn run(options: &[ResolvedOption<'_>]) -> Vec<MALAnime> {
    if let Some(ResolvedOption {
        value: ResolvedValue::String(anime),
        ..
    }) = options.first()
    {
        let response = request(anime).await.unwrap();
        response
    } else {
        panic!("please provide a valid value")
    }
}

pub async fn request(anime: &str) -> Result<Vec<MALAnime>, reqwest::Error> {
    let endpoint = format!("https://api.jikan.moe/v4/anime?q={}", anime);
    let response = reqwest::get(&endpoint)
        .await?
        .json::<serde_json::Value>()
        .await?;

    println!("API called: {}", &endpoint);
    let mal_data = response["data"].as_array().unwrap();

    let mal_anime: Vec<MALAnime> = mal_data
        .iter()
        .map(|x| MALAnime {
            id: x["mal_id"].as_i64().unwrap(),
            title: x["title"].as_str().unwrap().to_string(),
        })
        .collect();

    Ok(mal_anime)
}

pub fn register() -> CreateCommand {
    CreateCommand::new("anime")
        .description("Rengo-chan will help you with your anime stuff! :))")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "search",
                "What do you want to see?",
            )
            .required(true),
        )
}
