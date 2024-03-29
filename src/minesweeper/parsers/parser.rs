use crate::minesweeper::minesweeper_logic::{Board, FieldState};
use serde::{Deserialize, Serialize};

pub trait Iparser {
    fn supported_versions(&self) -> Vec<&str>;
    fn parse_mine_data(&self, data: &str, metadata: &Metadata) -> Board;
    fn parse_mine_locations(&self, data: &str) -> Vec<(i32, i32)>;
    fn parse_flag_data(&self, data: &str) -> Vec<FlagAction>;
    fn parse_open_data(&self, data: &str) -> Vec<OpenAction>;
    fn parse_meta_data(&self, data: &str) -> Metadata;
}

#[derive(Serialize, Deserialize)]
pub struct ApiData {
    #[serde(rename = "gameData")]
    pub game_data: String,
}

#[derive(Debug, Clone)]
pub struct Metadata {
    pub x_size: i32,
    pub y_size: i32,
    pub timeunits: i32,
}

#[derive(Debug)]
pub struct FlagAction {
    pub x: i32,
    pub y: i32,
    pub time: i64,
    pub action: Action,
    pub total_time: i64,
}

#[derive(Debug)]
pub enum Action {
    Place,
    Remove,
    Toggle,
}

#[derive(Debug)]
pub struct OpenAction {
    pub x: i32,
    pub y: i32,
    pub time: i64,
    pub total_time: i64,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ActionType {
    Open,
    Flag,
}

pub struct ParsedData {
    pub metadata: Metadata,
    pub game_board: Board,
    pub open_data: Vec<OpenAction>,
    pub flag_data: Vec<FlagAction>,
}

impl FlagAction {
    pub(crate) fn perform_action(&self, board: &mut Board) {
        match self.action {
            Action::Place => {
                board.fields[self.y as usize][self.x as usize].field_state = FieldState::Flagged;
                board.changed_fields[self.y as usize][self.x as usize] = true;
            }
            Action::Remove => {
                board.fields[self.y as usize][self.x as usize].field_state = FieldState::Closed;
                board.changed_fields[self.y as usize][self.x as usize] = true;
            }
            Action::Toggle => {
                board.fields[self.y as usize][self.x as usize].field_state =
                    FieldState::UnsureFlagged;
                board.changed_fields[self.y as usize][self.x as usize] = true;
            }
        }
    }
}
