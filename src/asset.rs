use crate::asset_entry::AssetEntry;
use elements::AssetId;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Metadata {
    pub amp: Option<bool>,
    pub stablecoin: Option<bool>,
    pub weight: Option<u16>,
    pub pair: Option<String>,
    pub meme: Option<bool>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Asset {
    pub asset_id: AssetId,
    pub asset_entry: Option<AssetEntry>,
    pub supply: Option<String>,
    pub metadata: Option<Metadata>,
    pub icon: Option<String>,
}
impl Asset {
    pub fn filter(&self, text: &String) -> bool {
        if let Some(asset_entry) = self.asset_entry.as_ref() {
            return asset_entry.name.contains(text)
                || asset_entry
                    .ticker
                    .as_ref()
                    .unwrap_or(&"".to_string())
                    .contains(text)
                || asset_entry
                    .entity
                    .get("domain")
                    .filter(|x| x.as_str().unwrap_or_default().contains(text))
                    .is_some();
        } else {
            return false; //self.asset_id.to_string().contains(text)
        }
    }
    pub fn is_amp(&self) -> bool {
        self.metadata
            .as_ref()
            .is_some_and(|x| x.amp.unwrap_or(false))
    }
    pub fn is_stablecoin(&self) -> bool {
        self.metadata
            .as_ref()
            .is_some_and(|x| x.stablecoin.unwrap_or(false))
    }
    pub fn is_meme(&self) -> bool {
        self.metadata
            .as_ref()
            .is_some_and(|x| x.meme.unwrap_or(false))
    }
}
