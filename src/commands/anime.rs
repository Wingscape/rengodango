use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::model::application::{CommandOptionType, ResolvedOption, ResolvedValue};

pub async fn run(options: &[ResolvedOption<'_>]) -> String {
    if let Some(ResolvedOption {
        value: ResolvedValue::String(anime),
        ..
    }) = options.first()
    {
        match request(anime).await {
            Ok(response) => format!("{response}"),
            Err(e) => format!("error: {e}"),
        }
    } else {
        "please provide a valid value".to_string()
    }
}

pub async fn request(anime: &str) -> Result<String, reqwest::Error> {
    let endpoint = format!("https://api.jikan.moe/v4/anime?q={}", anime);
    let response = reqwest::get(&endpoint)
        .await?
        .json::<serde_json::Value>()
        .await?;

    println!("API called: {}", &endpoint);
    println!("response to user: {}", response["data"][0]["url"]);

    Ok(response["data"][0]["url"].to_string())
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
