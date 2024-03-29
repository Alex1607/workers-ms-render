use std::collections::HashMap;

use worker::*;

use crate::minesweeper::error::MinesweeperError;
use crate::minesweeper::parsers;
use crate::minesweeper::parsers::parser::{Iparser, ParsedData};
use crate::minesweeper::provider::greev::greev_provider::GreevProvider;
use crate::minesweeper::provider::mcplayhd::mcplay_provider::McPlayHdProvider;
use crate::minesweeper::provider::provider::EnumProviders::{Greev, McPlayHd};
use crate::minesweeper::provider::provider::{ApiData, EnumProviders, Provider};
use crate::minesweeper::renderer::Renderer;

mod minesweeper;

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let router = Router::new();

    router
        .get_async("/render/:provider/:gameid", |request, context| async move {
            let Some(game_id) = context.param("gameid") else {
                return Response::error("GameId Missing", 400);
            };

            let Some(provider) = context.param("provider") else {
                return Response::error("Provider Missing", 400);
            };

            let hash_query: HashMap<_, _> = request.url()?.query_pairs().into_owned().collect();
            let gif = hash_query
                .get("gif")
                .map(|x| x.parse::<bool>().unwrap_or(false))
                .unwrap_or(false);

            let possible_providers: Vec<EnumProviders> =
                vec![Greev(GreevProvider), McPlayHd(McPlayHdProvider)];

            let optional_provider = possible_providers.iter().find(|x| match x {
                Greev(x) => x.id() == provider.as_str(),
                McPlayHd(x) => x.id() == provider.as_str(),
            });

            let Some(provider) = optional_provider else {
                return Response::error("Unknown Provider", 400);
            };

            let mut options: HashMap<String, String> = HashMap::new();
            let mcplay_api_key = context.secret("MCPLAYHD_API_KEY");
            if let Ok(api_key) = mcplay_api_key {
                options.insert("api_key".to_string(), api_key.to_string());
            }

            let result_api_data = provider.fetch_data(game_id, Some(options)).await;

            if let Err(err) = result_api_data {
                return Response::error(
                    format!("Unable to fetch game data because of {}", err),
                    500,
                );
            }

            let api_data = result_api_data.unwrap();

            let image_data_result = get_image_data(&api_data, &gif).await;

            if image_data_result.is_err() {
                return Response::error("Unable to fetch image data", 500);
            }

            if let Some(data) = image_data_result.unwrap() {
                return Response::from_body(ResponseBody::Body(data));
            }

            Response::error("Unable to fetch image data", 500)
        })
        .run(req, env)
        .await
}

async fn get_image_data(
    api_data: &ApiData,
    mut gif: &bool,
) -> std::result::Result<Option<Vec<u8>>, MinesweeperError> {
    if let Some(game_data) = &api_data.game_data {
        let option = game_data.split_once('=').expect("Unable to get Version");

        let possible_parsers: Vec<&dyn Iparser> = vec![
            &parsers::v1::parser::ParserV1,
            &parsers::v2::parser::ParserV2,
        ];

        let option_found_parser = possible_parsers
            .iter()
            .find(|p| p.supported_versions().contains(&option.0));

        if option_found_parser.is_none() {
            return Err(MinesweeperError::UnsupportedVersion);
        }

        let parser = option_found_parser.unwrap();

        let split: Vec<&str> = option.1.split('+').collect();

        let metadata = parser.parse_meta_data(split[0].trim());

        let game_data = ParsedData {
            game_board: parser.parse_mine_data(split[1].trim(), &metadata),
            open_data: parser.parse_open_data(split[2].trim()),
            flag_data: parser.parse_flag_data(split[3].trim()),
            metadata,
        };

        //If the field is too large overwrite the gif value to not render a gif
        if game_data.metadata.x_size > 32 || game_data.metadata.y_size > 32 {
            gif = &false
        }

        let mut renderer = Renderer::new(
            game_data.metadata,
            game_data.game_board,
            game_data.open_data,
            game_data.flag_data,
            gif,
        );

        Ok(Some(if *gif {
            renderer
                .render_gif()
                .map_err(|_| MinesweeperError::ImageRender)?
        } else {
            renderer
                .render_jpeg()
                .map_err(|_| MinesweeperError::ImageRender)?
        }))
    } else {
        Ok(None)
    }
}
