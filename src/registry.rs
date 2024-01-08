use elements::AssetId;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Error;

use crate::asset::{Asset, Metadata};
use crate::asset_entry::AssetEntry;


#[derive(Deserialize, Debug)]
struct BinancePrice {
    price: String
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
    Iconed,
    Text(String),
}
impl Registry {
    pub fn new() -> Self {
        let mut registry = Registry {
            assets: IndexMap::default(),
        };
        registry.assets = registry.download().unwrap();
        registry
    }

    pub fn download(&mut self) -> Result<IndexMap<AssetId, Asset>, Error> {
        let asset_entries = Self::get_assets().unwrap();
        let metadatas = Self::get_metadata().unwrap();
        let icons = Self::get_icons().unwrap();
        let mut assets = IndexMap::default();

        for asset in asset_entries {
            assets.insert(
                asset.0,
                Asset {
                    asset_id: asset.0,
                    asset_entry: Some(asset.1),
                    icon: icons.get(&asset.0).cloned(),
                    metadata: metadatas.get(&asset.0).cloned(),
                    supply: None,
                },
            );
        }
        Ok(assets)
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
    pub async fn query_by_id(&self, id: AssetId) -> Result<&Asset, Error> {
        Ok(self.assets.get(&id).unwrap())
    }
    pub async fn query_by_ids(&self, ids: Vec<AssetId>) -> Result<Vec<&Asset>, Error> {
        Ok(ids.iter().filter_map(|i| self.assets.get(i)).collect())
    }
    pub async fn query(&self, filter: Filter) -> Result<Vec<AssetId>, Error> {
        match filter {
            Filter::All => Ok(self.assets.values().map(|x| x.asset_id).collect()),
            Filter::Main => Ok(self
                .assets
                .values()
                .filter(|x| {
                    x.metadata
                        .as_ref()
                        .is_some_and(|x| x.weight.is_some_and(|x| x > 0))
                })
                .map(|x| x.asset_id)
                .collect()),
            Filter::Amp => Ok(self
                .assets
                .values()
                .filter(|x| {
                    x.metadata
                        .as_ref()
                        .is_some_and(|x| x.amp.is_some_and(|x| x))
                })
                .map(|x| x.asset_id)
                .collect()),
            Filter::Iconed => Ok(self
                .assets
                .values()
                .filter(|x| {
                    x.icon
                        .as_ref()
                        .is_some()
                })
                .map(|x| x.asset_id)
                .collect()),
            Filter::Stablecoins => Ok(self
                .assets
                .values()
                .filter(|x| {
                    x.metadata
                        .as_ref()
                        .is_some_and(|x| x.stablecoin.as_ref().is_some_and(|x| *x == true))
                })
                .map(|x| x.asset_id)
                .collect()),
            Filter::Text(text) => Ok(self
                .assets
                .values()
                .filter(|x| x.filter(&text))
                .map(|x| x.asset_id)
                .collect()),
        }
    }

    pub async fn fetch(&self, asset_id: AssetId) -> Result<AssetEntry, Error> {
        let url = format!("https://blockstream.info/liquid/api/asset/{}", asset_id);
        let res = reqwest::get(url).await.unwrap();
        let asset: AssetEntry = res.json().await.unwrap();
        println!("Asset: {:#?}", asset);
        Ok(asset)
    }

    pub async fn supply(&self, asset_id: AssetId) -> Result<String, Error> {
        let url = format!(
            "https://blockstream.info/liquid/api/asset/{}/supply/decimal",
            asset_id
        );
        let res = reqwest::get(url).await.unwrap();
        let supply = res.text().await.unwrap();
        println!("Supply: {:#?}", supply);
        Ok(supply)
    }

    pub async fn price(&self, asset_id: AssetId) -> Result<String, Error> {
        let pair = self.assets
            .get(&asset_id)
            .and_then(|a| a.metadata.as_ref().and_then(|x| x.pair.as_ref()));
        match pair {
            Some(pair) => {
                let url = format!(
                    "https://api.binance.com/api/v3/avgPrice?symbol={}", 
                    pair
                );
                let res = reqwest::get(url).await.unwrap();
                let price = res.json::<BinancePrice>().await.unwrap();
                println!("Price: {:#?}", price.price);
                return Ok(price.price)
            },
            None => Err(serde::de::Error::missing_field("")),
        }
    }
}
