use bytes::Bytes;
use regex::Regex;
use serde::Deserialize;
use sqlx::FromRow;
use std::io::Write;
use std::{error::Error, fs::File};

#[derive(Debug, Clone, Deserialize, FromRow)]
pub struct AssetDatabase {
    pub hash: String,
}

#[derive(Clone, Debug)]
pub struct Asset {
    pub hash: String,
    pub file_type: String,
    pub path: String,
}

impl Asset {
    pub fn get_discord_asset_url(&self) -> String {
        const BASE_URL: &str = "https://canary.discord.com/assets/";
        format!("{}{}", BASE_URL, self.path)
    }

    fn get_discord_url(&self) -> String {
        const BASE_URL: &str = "https://canary.discord.com";
        format!("{}{}", BASE_URL, self.path)
    }

    pub async fn get_all_assets(&self) -> Result<Vec<Asset>, Box<dyn Error>> {
        let asset_rg = Regex::new(r"([+_a-zA-Z0-9]{32})\.(svg|png|gif|mp3|mp4|jpg|ico|mov|webm)")?;

        let asset_file_types: Vec<&str> = vec![
            "png", "svg", "ico", "gif", "mp3", "mp4", "jpg", "ico", "mov", "webm",
        ];
        if asset_file_types.contains(&self.file_type.as_str()) {
            return Ok(vec![Asset {
                path: format!("{}.{}", &self.hash, &self.file_type),
                hash: self.hash.clone(),
                file_type: self.file_type.clone(),
            }]);
        }

        let mut assets: Vec<Asset> = vec![];

        let resp = reqwest::get(self.get_discord_url()).await?;
        let txt = resp.text().await?;

        for file in asset_rg.captures_iter(&txt) {
            let file_asset = Asset {
                path: file.get(0).unwrap().as_str().to_string(),
                hash: file.get(1).unwrap().as_str().to_string(),
                file_type: file.get(2).unwrap().as_str().to_string(),
            };

            assets.push(file_asset);
        }

        Ok(assets)
    }

    pub async fn download(&self) -> Result<Bytes, Box<dyn Error>> {
        let path = format!("./assets/{}.{}", self.hash, self.file_type);

        let img_bytes = reqwest::get(&self.get_discord_asset_url())
            .await?
            .bytes()
            .await?;

        let mut file = File::create(&path)?;
        file.write_all(&img_bytes)?;

        Ok(img_bytes)
    }
}
