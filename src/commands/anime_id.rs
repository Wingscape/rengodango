pub struct MALAnimeID {
    pub id: i64,
    pub title: String,
    pub url: String,
    pub image: String,
    pub synopsis: String,
}

pub async fn run(id: &str) -> MALAnimeID {
    let response = request(id).await.unwrap();
    response
}

pub async fn request(id: &str) -> Result<MALAnimeID, reqwest::Error> {
    let endpoint = format!("https://api.jikan.moe/v4/anime/{}", id);
    let response = reqwest::get(&endpoint)
        .await?
        .json::<serde_json::Value>()
        .await?;

    println!("API called: {}", &endpoint);

    let mal_data = MALAnimeID {
        id: response["data"]["mal_id"].as_i64().unwrap(),
        title: response["data"]["title"].as_str().unwrap().to_string(),
        url: response["data"]["url"].as_str().unwrap().to_string(),
        image: response["data"]["images"]["webp"]["image_url"]
            .as_str()
            .unwrap()
            .to_string(),
        synopsis: response["data"]["synopsis"].as_str().unwrap().to_string(),
    };

    Ok(mal_data)
}
