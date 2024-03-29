use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use worker::{Fetch, Headers, Method, Request, RequestInit, RequestRedirect};

use crate::minesweeper::base36;
use crate::minesweeper::error::MinesweeperError;
use crate::minesweeper::provider::provider::{ApiData, Provider};

pub struct McPlayHdProvider;

impl Provider for McPlayHdProvider {
    fn id(&self) -> &str {
        "mcplayhd"
    }

    fn name(&self) -> &str {
        "McPlayHD"
    }

    async fn fetch_data(
        &self,
        game_id: &str,
        options: Option<HashMap<String, String>>,
    ) -> Result<ApiData, MinesweeperError> {
        let Some(options) = options else {
            return Err(MinesweeperError::ApiKeyNotFound);
        };

        let api_key = options.get("api_key");
        if api_key.is_none() || String::is_empty(api_key.unwrap()) {
            return Err(MinesweeperError::ApiKeyNotFound);
        }

        let id = base36::decode(game_id);

        let mut headers = Headers::new();
        headers
            .set(
                "Authorization",
                format!("Bearer {}", api_key.unwrap()).as_str(),
            )
            .expect("Failed to set header");

        let mut request_init = RequestInit::new();
        request_init
            .with_method(Method::Get)
            .with_headers(headers)
            .with_redirect(RequestRedirect::Follow);

        let new_request = Request::new_with_init(
            format!("https://mcplayhd.net/api/v1/minesweeper/game/{id}").as_str(),
            &request_init,
        );

        let Ok(request) = new_request else {
            return Err(MinesweeperError::GameDataNotFound);
        };

        let response = Fetch::Request(request).send().await;
        let ms_data = response
            .unwrap()
            .json::<Response>()
            .await
            .map_err(|_| MinesweeperError::ApiDataParse)?;

        Ok(ApiData {
            game_data: Some(ms_data.data.game_info.algebraic_notation.clone()),
            tiepe: None,
            time: ms_data.data.game_info.time_taken,
            generator: None,
            uuid: ms_data.data.game_info.uuid.clone(),
            correct_flags: Some(ms_data.data.game_info.flags_correct),
            incorrect_flags: Some(ms_data.data.game_info.flags_incorrect),
            won: ms_data.data.game_info.won,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct GameInfo {
    id: u32,
    uuid: String,
    won: bool,
    #[serde(rename = "flagsCorrect")]
    flags_correct: u32,
    #[serde(rename = "flagsIncorrect")]
    flags_incorrect: u32,
    #[serde(rename = "timeStart")]
    time_start: u64,
    #[serde(rename = "timeEnd")]
    time_end: u64,
    #[serde(rename = "timeTaken")]
    time_taken: u64,
    mines: u32,
    #[serde(rename = "sizeX")]
    size_x: u32,
    #[serde(rename = "sizeZ")]
    size_z: u32,
    #[serde(rename = "algebraicNotation")]
    algebraic_notation: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Player {
    uuid: String,
    name: String,
    group: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Data {
    #[serde(rename = "gameInfo")]
    game_info: GameInfo,
    players: Vec<Player>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Response {
    status: u32,
    data: Data,
}
