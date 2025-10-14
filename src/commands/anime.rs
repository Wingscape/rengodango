use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::model::application::{CommandOptionType, ResolvedOption, ResolvedValue};

pub struct MALAnime {
    pub mal_id: i64,
    pub mal_title: String,
}

pub async fn run(options: &[ResolvedOption<'_>]) -> MALAnime {
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

pub async fn request(anime: &str) -> Result<MALAnime, reqwest::Error> {
    let endpoint = format!("https://api.jikan.moe/v4/anime?q={}", anime);
    let response = reqwest::get(&endpoint)
        .await?
        .json::<serde_json::Value>()
        .await?;

    let response_api_url = response["data"][0]["url"].as_str().unwrap();

    println!("API called: {}", &endpoint);
    println!("response to user: {}", response_api_url);

    let mal_anime = MALAnime {
        mal_id: response["data"][0]["mal_id"].as_i64().unwrap(),
        mal_title: response["data"][0]["titles"][0]["title"]
            .as_str()
            .unwrap()
            .to_string(),
    };

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
