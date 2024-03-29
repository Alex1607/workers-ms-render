use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use worker::Method::Get;
use worker::{Fetch, Request};

use crate::minesweeper::error::MinesweeperError;
use crate::minesweeper::provider::greev::greev_provider::GreevProvider;
use crate::minesweeper::provider::mcplayhd::mcplay_provider::McPlayHdProvider;

pub trait Provider {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    async fn fetch_data(
        &self,
        game_id: &str,
        options: Option<HashMap<String, String>>,
    ) -> Result<ApiData, MinesweeperError>;
    async fn fetch_name(&self, _uuid: &str) -> Result<PlayerData, MinesweeperError> {
        unreachable!()
    }
}

pub enum EnumProviders {
    Greev(GreevProvider),
    McPlayHd(McPlayHdProvider),
}

impl Provider for EnumProviders {
    fn id(&self) -> &str {
        match self {
            EnumProviders::Greev(provider) => provider.id(),
            EnumProviders::McPlayHd(provider) => provider.id(),
        }
    }

    fn name(&self) -> &str {
        match self {
            EnumProviders::Greev(provider) => provider.name(),
            EnumProviders::McPlayHd(provider) => provider.name(),
        }
    }

    async fn fetch_data(
        &self,
        game_id: &str,
        options: Option<HashMap<String, String>>,
    ) -> Result<ApiData, MinesweeperError> {
        match self {
            EnumProviders::Greev(provider) => provider.fetch_data(game_id, None).await,
            EnumProviders::McPlayHd(provider) => provider.fetch_data(game_id, options).await,
        }
    }

    async fn fetch_name(&self, uuid: &str) -> Result<PlayerData, MinesweeperError> {
        let new_request = Request::new(
            format!("https://api.greev.eu/v2/player/name/{uuid}").as_str(),
            Get,
        );
        let Ok(request) = new_request else {
            return Err(MinesweeperError::GameDataNotFound);
        };
        let response = Fetch::Request(request).send().await;
        response
            .unwrap()
            .json::<PlayerData>()
            .await
            .map_err(|_| MinesweeperError::ApiDataParse)
    }
}

#[derive(Serialize, Deserialize)]
pub struct ApiData {
    #[serde(rename = "gameData")]
    pub game_data: Option<String>,
    #[serde(rename = "type")]
    pub tiepe: Option<String>,
    pub time: u64,
    pub generator: Option<String>,
    pub uuid: String,
    #[serde(rename = "correctFlags")]
    pub correct_flags: Option<u32>,
    #[serde(rename = "incorrectFlags")]
    pub incorrect_flags: Option<u32>,
    pub won: bool,
}

#[derive(Serialize, Deserialize)]
pub struct PlayerData {
    pub name: String,
}
