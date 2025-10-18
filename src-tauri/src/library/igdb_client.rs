use crate::library::{Game, GameMetadata};
use anyhow::Result;
use reqwest::Client as ReqwestClient;
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;

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

    async fn multiquery<T>(&self, fields: &[&str], games: &[u32]) -> Result<Vec<T>>
    where
        T: serde::de::DeserializeOwned + Clone,
    {
        if games.is_empty() {
            return Ok(vec![]);
        }
        let query = format!(
            "query games \"Get Games Infos\" {{ fields {}; where id = ({}); limit 500; }};",
            fields.join(", "),
            games
                .into_iter()
                .map(|game| game.to_string())
                .collect::<Vec<_>>()
                .join(", "),
        );
        println!("query: {query}");
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

    pub async fn load_igdb_games(&mut self, games: &Vec<Game>) -> Result<()> {
        let igdb_ids: Vec<u32> = games
            .iter()
            .filter_map(|game| match game.metadata.igdbid {
                Some(id) => game.metadata.igdbid,
                _ => None,
            })
            .collect();
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
        let igdb_games: Vec<IgdbGameInfos> = self.multiquery(&fields, &igdb_ids).await?;
        for igdb_game in igdb_games {
            self.cache.insert(igdb_game.id, igdb_game);
        }
        Ok(())
    }

	pub fn fill_game_metadata(&self, meta: &mut GameMetadata) {
		if let Some(igdbid) = meta.igdbid {
			if let Some(game_info) = self.cache.get(&igdbid) {
				let data = game_info.clone();
                

                meta.cover = data.cover.map(|cover| {
                    format!(
                        "https://images.igdb.com/igdb/image/upload/t_{}/{}.jpg",
                        "cover_big_2x", cover.image_id
                    )
                });

                meta.hero = data
                    .artworks
                    .and_then(|mut artworks| {
                        if !artworks.is_empty() {
                            Some(artworks.remove(0))
                        } else {
                            None
                        }
                    })
                    .map(|artwork| artwork.clone())
                    .map(|artwork| {
                        format!(
                            "https://images.igdb.com/igdb/image/upload/t_{}/{}.jpg",
                            "1080p_2x", artwork.image_id
                        )
                    });

                meta.screenshots = data.screenshots.map(|screenshots| {
                    screenshots
                        .into_iter()
                        .map(|screenshot| {
                            format!(
                                "https://images.igdb.com/igdb/image/upload/t_{}/{}.jpg",
                                "1080p_2x", screenshot.image_id
                            )
                        })
                        .collect()
                });

                meta.movies = data
                    .videos
                    .map(|videos| videos.into_iter().map(|video| video.video_id).collect());

                meta.store_pages = None; // dont know what this should be

                meta.icon = None;
                meta.logo = None;
                meta.short_description = None;
                meta.movies_thumbnails = None;
                meta.tags = None;
            }
        }
    }
}
