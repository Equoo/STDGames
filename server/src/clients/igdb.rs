use anyhow::Result;
use reqwest::Client as ReqwestClient;
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;
use tracing::{debug, info};

use crate::clients::{ApiClient, GameAssetList, GameMetadata};

#[derive(Deserialize)]
struct IgdbApiAuthResponse {
    access_token: String,
    expires_in: u32,
    token_type: String,
}

#[derive(Deserialize, Clone)]
struct IgdbApiMultiqueryResponse<T> {
    name: String,
    result: Vec<T>,
}

#[derive(Deserialize, Clone)]
struct IgdbCover {
    image_id: String,
}

#[derive(Deserialize, Clone)]
struct IgdbGenres {
    slug: String,
    name: String,
}

#[derive(Deserialize, Clone)]
struct IgdbArtworks {
    image_id: String,
}

#[derive(Deserialize, Clone)]
struct IgdbScreenshots {
    image_id: String,
}

#[derive(Deserialize, Clone)]
struct IgdbVideos {
    video_id: String,
}

#[derive(Deserialize, Clone)]
struct IgdbWebsite {
    url: String,
}

#[derive(Deserialize, Clone)]
struct IgdbGameInfos {
    id: u32,
    name: String,
    slug: String,
    summary: String,
    cover: Option<IgdbCover>,
    genres: Option<Vec<IgdbGenres>>,
    artworks: Option<Vec<IgdbArtworks>>,
    screenshots: Option<Vec<IgdbScreenshots>>,
    videos: Option<Vec<IgdbVideos>>,
    websites: Option<Vec<IgdbWebsite>>,
}

pub struct IgdbClient {
    reqwest_client: ReqwestClient,
    client_id: String,
    bearer_token: String,
    cache: HashMap<u32, IgdbGameInfos>,
}

impl IgdbClient {
    pub async fn new(client_id: &str, client_secret: &str) -> Result<Self> {
        let client = ReqwestClient::new();
        let res = client
            .post("https://id.twitch.tv/oauth2/token")
            .query(&[
                ("client_id", &client_id),
                ("client_secret", &client_secret),
                ("grant_type", &"client_credentials"),
            ])
            .send()
            .await?
            .error_for_status()?;
        let token = res.json::<IgdbApiAuthResponse>().await?.access_token;
        Ok(Self {
            reqwest_client: client,
            client_id: client_id.to_string(),
            bearer_token: token,
            cache: HashMap::new(),
        })
    }

    async fn multiquery<T>(&self, fields: &[&str], game: u32) -> Result<Vec<T>>
    where
        T: serde::de::DeserializeOwned + Clone,
    {
        let query = format!(
            "query games \"Get Games Infos\" {{ fields {}; where id = ({game}); limit 500; }};",
            fields.join(", "),
        );
        debug!("query: {query}");
        let res = self
            .reqwest_client
            .post("https://api.igdb.com/v4/multiquery")
            .header("Accept", "application/json")
            .header("Client-ID", &self.client_id)
            .bearer_auth(&self.bearer_token)
            .body(query.to_string())
            .send()
            .await?
            .error_for_status()?;
        let data = res.json::<[IgdbApiMultiqueryResponse<T>; 1]>().await?;
        return Ok(data[0].result.clone());
    }

    pub async fn load_igdb_game(&mut self, game: u32) -> Result<IgdbGameInfos> {
        let fields = [
            "slug",
            "name",
            "summary",
            "genres.slug",
            "genres.name",
            "cover.image_id",
            "artworks.image_id",
            "screenshots.image_id",
            "videos.video_id",
            "websites.url",
            "websites.type",
            // "cover.width", "cover.height", "videos.name",
            // "artworks.width", "artworks.height",
            // "screenshots.width", "screenshots.height",
        ];
        let igdb_games: Vec<IgdbGameInfos> = self.multiquery(&fields, game).await?;
        Ok(igdb_games[0].clone())
    }

    pub async fn fetch_download_data(
        &mut self,
        resources_path: &PathBuf,
        app_id: u32,
        lang: &str,
    ) -> Option<GameMetadata> {
        let mut assets = GameAssetList::new();
        let data = self.load_igdb_game(app_id).await.ok()?;

        let cover = data.cover.map(|cover| {
            assets.add_asset(
                format!(
                    "https://images.igdb.com/igdb/image/upload/t_{}/{}.jpg",
                    "cover_big_2x", cover.image_id
                ),
                resources_path.join("cover.jpg"),
            )
        });

        let hero = data.artworks.and_then(|artworks| {
            artworks.get(0).map(|artwork| {
                assets.add_asset(
                    format!(
                        "https://images.igdb.com/igdb/image/upload/t_{}/{}.jpg",
                        "1080p_2x", artwork.image_id
                    ),
                    resources_path.join("library_hero.jpg"),
                )
            })
        });

        let screenshots = data.screenshots.map(|screenshots| {
            screenshots
                .iter()
                .map(|screenshot| {
                    assets.add_asset(
                        format!(
                            "https://images.igdb.com/igdb/image/upload/t_{}/{}.jpg",
                            "1080p_2x", screenshot.image_id
                        ),
                        resources_path.join(format!("{}.jpg", screenshot.image_id)),
                    )
                })
                .collect()
        });

        let movies = data.videos.map(|videos| {
            videos
                .iter()
                .map(|video| {
                    assets.add_asset(
                        format!("https://www.youtube.com/watch?v={}", video.video_id),
                        resources_path.join(format!("{}.mp4", video.video_id)),
                    )
                })
                .collect()
        });

        let tags = data
            .genres
            .map(|genres| genres.iter().map(|v| v.name.clone()).collect());

        let metadata = GameMetadata {
            store_pages: data
                .websites
                .map(|wbsts| wbsts.iter().map(|v| v.url.clone()).collect()),
            name: Some(data.name.clone()),
            icon: None,
            logo: None,
            hero: hero,
            cover: cover,
            description: Some(data.summary.clone()),
            short_description: None,
            screenshots: screenshots,
            movies: movies,
            movies_thumbnails: None,
            tags: tags,
            assets: assets.get_asset_paths(),
        };

        assets.download_all().await.ok()?;

        let metadata_path = resources_path.join("data.json");
        let metadata_json = serde_json::to_string_pretty(&metadata).ok()?;
        tokio::fs::write(metadata_path, metadata_json).await.ok()?;

        Some(metadata)
    }
}
