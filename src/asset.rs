use elements::AssetId;
use serde_json::Error;

use crate::asset_entry::AssetEntry;
use indexmap::IndexMap;
use std::fs::File;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Metadata {
    pub amp: Option<bool>,
    pub category: Option<String>,
    pub weight: Option<u16>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Asset {
    pub asset_id: AssetId,
    pub asset_entry: Option<AssetEntry>,
    pub supply: Option<String>,
    pub metadata: Option<Metadata>,
    pub icon: Option<String>,
}
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Registry {
    pub assets: IndexMap<AssetId, Asset>,
}
pub enum Filter {
    All,
    Main,
    Amp,
    Stablecoins,
    Text(String),
}
impl Registry {
    pub fn new() -> Self {
        let mut registry = Registry {
            assets: IndexMap::default(),
        };
        registry.reload();
        registry
    }

    pub fn reload(&mut self) {
        for asset in Self::get_assets().unwrap() {
            self.assets.insert(
                asset.0,
                Asset {
                    asset_id: asset.0,
                    asset_entry: Some(asset.1),
                    icon: None,
                    metadata: None,
                    supply: None,
                },
            );
        }
        for metadata in Self::get_metadata().unwrap() {
            if let Some(asset) = self.assets.get_mut(&metadata.0) {
                asset.metadata = Some(metadata.1)
            }
        }
        for icon in Self::get_icons().unwrap() {
            if let Some(asset) = self.assets.get_mut(&icon.0) {
                asset.icon = Some(icon.1)
            }
        }
    }

    fn get_assets() -> Result<IndexMap<AssetId, AssetEntry>, Error> {
        let content = std::include_str!("../assets/liquid_assets.json");
        serde_json::from_str(content)
    }

    fn get_icons() -> Result<IndexMap<AssetId, String>, Error> {
        let content = std::include_str!("../assets/liquid_icons.json");
        serde_json::from_str(content)
    }

    fn get_metadata() -> Result<IndexMap<AssetId, Metadata>, Error> {
        let content = std::include_str!("../assets/liquid_metadatas.json");
        serde_json::from_str(content)
    }
}

impl Registry {
    pub async fn query(&self, filter: Filter) -> Result<Vec<&Asset>, Error> {
        match filter {
            Filter::All => Ok(self.assets.values().collect()),
            Filter::Main => Ok(self
                .assets
                .values()
                .filter(|x| {
                    x.metadata
                        .as_ref()
                        .is_some_and(|x| x.weight.is_some_and(|x| x > 0))
                })
                .collect()),
            Filter::Amp => Ok(self
                .assets
                .values()
                .filter(|x| {
                    x.metadata
                        .as_ref()
                        .is_some_and(|x| x.amp.is_some_and(|x| x))
                })
                .collect()),
            Filter::Stablecoins => Ok(self
                .assets
                .values()
                .filter(|x| {
                    x.metadata
                        .as_ref()
                        .is_some_and(|x| x.category.as_ref().is_some_and(|x| x == "stablecoin"))
                })
                .collect()),
            Filter::Text(text) => {
                fn filtering(asset: &Asset, text: &String) -> bool {
                    asset.asset_id.to_string().contains(text) ||
                    asset.asset_entry.as_ref().is_some_and(|x| x.name.contains(text)) ||
                    asset.asset_entry.as_ref().is_some_and(|x| x.ticker.as_ref().unwrap_or(&"".to_string()).contains(text)) ||
                    //asset.ticker.unwrap_or("".to_string()).contains(text) ||
                    asset.asset_entry.as_ref().is_some_and(|x| x.entity.get("domain").filter(|x| x.as_str().unwrap_or_default().contains(text)).is_some())
                }
                Ok(self
                    .assets
                    .values()
                    .filter(|x| filtering(x, &text))
                    .collect())
            }
        }
    }

    pub async fn fetch(&mut self, asset_id: AssetId) -> Result<AssetEntry, Error> {
        let url = format!("https://blockstream.info/liquid/api/asset/{}", asset_id);
        let res = reqwest::get(url).await.unwrap();
        let asset: AssetEntry = res.json().await.unwrap();
        println!("Asset: {:#?}", asset);
        Ok(asset)
    }

    pub async fn supply(&mut self, asset_id: AssetId) -> Result<String, Error> {
        let url = format!(
            "https://blockstream.info/liquid/api/asset/{}/supply/decimal",
            asset_id
        );
        let res = reqwest::get(url).await.unwrap();
        let supply = res.text().await.unwrap();
        println!("Supply: {:#?}", supply);
        Ok(supply)
    }
}
