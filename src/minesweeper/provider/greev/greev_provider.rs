use std::collections::HashMap;
use worker::Method::Get;
use worker::{Fetch, Request};

use crate::minesweeper::error::MinesweeperError;
use crate::minesweeper::provider::provider::{ApiData, Provider};

pub struct GreevProvider;

impl Provider for GreevProvider {
    fn id(&self) -> &str {
        "greev"
    }

    fn name(&self) -> &str {
        "Greev"
    }

    async fn fetch_data(
        &self,
        gameid: &str,
        _: Option<HashMap<String, String>>,
    ) -> Result<ApiData, MinesweeperError> {
        let new_request = Request::new(
            format!("https://api.greev.eu/v2/stats/minesweeper/game/{gameid}").as_str(),
            Get,
        );
        let Ok(request) = new_request else {
            return Err(MinesweeperError::GameDataNotFound);
        };
        let response = Fetch::Request(request).send().await;
        return response
            .unwrap()
            .json::<ApiData>()
            .await
            .map_err(|_| MinesweeperError::ApiDataParse);
    }
}
