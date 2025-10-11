use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::model::application::{CommandOptionType, ResolvedOption, ResolvedValue};

pub async fn run(options: &[ResolvedOption<'_>]) -> String {
    if let Some(ResolvedOption {
        value: ResolvedValue::String(anime),
        ..
    }) = options.first()
    {
        match request_api(anime).await {
            Ok(resp) => format!("{resp}").to_string(),
            Err(e) => format!("Error: {e}").to_string(),
        }
    } else {
        "No".to_string()
    }
}

pub async fn request_api(anime: &str) -> Result<String, reqwest::Error> {
    let endpoint = format!("https://api.jikan.moe/v4/anime?q={}", anime);
    let api_response = reqwest::get(endpoint)
        .await?
        .json::<serde_json::Value>()
        .await?;

    println!("{}", api_response["data"][0]["url"]);
    Ok(api_response["data"][0]["url"].to_string())
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
