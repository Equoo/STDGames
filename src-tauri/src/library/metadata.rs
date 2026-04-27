
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GameMetadataAPI {
    pub store_pages: Option<Vec<String>>,
    pub name: Option<String>,
    pub icon: Option<usize>,
    pub logo: Option<usize>,
    pub hero: Option<usize>,
    pub cover: Option<usize>,
    pub description: Option<String>,
    pub short_description: Option<String>,
    pub screenshots: Option<Vec<usize>>,
    pub movies: Option<Vec<usize>>,
    pub movies_thumbnails: Option<Vec<usize>>,
    pub tags: Option<Vec<String>>,
    pub assets: Vec<String>,
}

fn get_asset(asset_id: Option<usize>, assets: &Vec<String>) -> Option<String> {
    match asset_id {
        Some(id) if id < assets.len() => Some(assets[id].clone()),
        _ => None,
    }
}

pub async fn load_api_data(games: &mut Vec<Game>) -> Result<()> {
    let client = reqwest::Client::new();

    println!("Sending API request...");
    let res = client
        .post("http://37.59.106.4:2356/api/data")
        .json(
            &games
                .iter()
                .filter_map(|g| {
                    g.metadata
                        .api
                        .as_ref()
                        .map(|api| (g.slug.clone(), api).into())
                })
                .collect::<HashMap<String, &ApiClient>>(),
        )
        .send()
        .await?;

    println!("API response status: {}", res.status());

    let mut api_data: HashMap<String, GameMetadataAPI> = res.json().await?;

    for game in games.iter_mut() {
        if let Some(data) = api_data.get_mut(&game.slug) {
            data.assets.iter_mut().for_each(|asset| {
                *asset = asset.replace("resources/", "http://37.59.106.4:2356/cdn/");
            });

            game.metadata.name = game.metadata.name.clone().or(data.name.clone());
            game.metadata.description = game
                .metadata
                .description
                .clone()
                .or(data.description.clone());
            game.metadata.short_description = game
                .metadata
                .short_description
                .clone()
                .or(data.short_description.clone());
            game.metadata.icon = game
                .metadata
                .icon
                .clone()
                .or(get_asset(data.icon, &data.assets));
            game.metadata.logo = game
                .metadata
                .logo
                .clone()
                .or(get_asset(data.logo, &data.assets));
            game.metadata.hero = game
                .metadata
                .hero
                .clone()
                .or(get_asset(data.hero, &data.assets));
            game.metadata.cover = game
                .metadata
                .cover
                .clone()
                .or(get_asset(data.cover, &data.assets));
            game.metadata.screenshots =
                game.metadata
                    .screenshots
                    .clone()
                    .or(match &data.screenshots {
                        Some(screenshots) => Some(
                            screenshots
                                .iter()
                                .map(|id| data.assets[*id].clone())
                                .collect(),
                        ),
                        _ => None,
                    });
            game.metadata.movies = game.metadata.movies.clone().or(match &data.movies {
                Some(movies) => Some(movies.iter().map(|id| data.assets[*id].clone()).collect()),
                _ => None,
            });
            game.metadata.movies_thumbnails =
                game.metadata
                    .movies_thumbnails
                    .clone()
                    .or(match &data.movies_thumbnails {
                        Some(thumbnails) => Some(
                            thumbnails
                                .iter()
                                .map(|id| data.assets[*id].clone())
                                .collect(),
                        ),
                        _ => None,
                    });
            game.metadata.tags = game.metadata.tags.clone().or(data.tags.clone());
        }
    }

    Ok(())
}


