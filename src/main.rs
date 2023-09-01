mod database;
mod models;

use crate::database::Database;
use crate::models::Asset;
use dotenv::dotenv;
use regex::Regex;
use std::error::Error;
use tokio::main;

#[main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let urls: Vec<&str> = vec![
        "https://canary.discord.com",
        "https://canary.discord.com/app",
    ];

    let asset_files_rg = Regex::new(r"\/assets\/([\.a-zA-Z0-9]+)\.([a-zA-Z0-9]+)")?;

    let mut found_assets: Vec<Asset> = vec![];

    for url in urls {
        let resp = reqwest::get(url).await?;
        let txt = resp.text().await?;

        for file in asset_files_rg.captures_iter(&txt) {
            let hash = file.get(1).unwrap().as_str().to_string();
            let file_type = file.get(2).unwrap().as_str().to_string();

            let file_asset = Asset {
                path: format!("/assets/{}.{}", hash, file_type),
                hash,
                file_type,
            };

            found_assets.push(file_asset);
        }
    }

    println!("[!] Found {} asset path files", found_assets.len());

    let mut database = Database::init().await?;
    database.setup_tables().await?;

    for asset_files in found_assets {
        let assets = asset_files.get_all_assets().await?;

        for asset in assets {
            if database.is_hash_in_db(&asset.hash).await {
                //database.add_hash_to_db(&asset.hash).await?;

                let bytes = asset.download().await.unwrap();
                println!("{:#?}", bytes);
            }
        }
    }

    Ok(())
}
