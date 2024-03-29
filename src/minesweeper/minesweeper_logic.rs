use crate::minesweeper::parsers::parser::Metadata;

#[derive(Debug)]
pub struct Board {
    pub fields: Vec<Vec<Field>>,
    pub changed_fields: Vec<Vec<bool>>,
    pub metadata: Metadata,
    pub open_fields: u32,
    pub mine_count: u32,
    pub total_fields: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Field {
    pub value: u8,
    pub field_state: FieldState,
    pub mine: bool,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum FieldState {
    Open,
    Closed,
    Flagged,
    UnsureFlagged,
}

impl Board {
    pub(crate) fn open_field(&mut self, x: usize, y: usize) {
        let field = &mut self.fields[y][x];

        //If flagged or already open return
        if field.field_state != FieldState::Closed {
            return;
        }

        field.field_state = FieldState::Open;
        self.changed_fields[y][x] = true;
        self.open_fields += 1;

        if field.mine {
            return;
        }

        if field.value == 0 {
            for xd in -1..=1_i32 {
                for yd in -1..=1_i32 {
                    let xx = xd + x as i32;
                    let yy = yd + y as i32;
                    if xx < 0
                        || xx >= self.metadata.x_size
                        || yy < 0
                        || yy >= self.metadata.y_size
                        || xd == 0 && yd == 0
                    {
                        continue;
                    }
                    self.open_field(xx as usize, yy as usize)
                }
            }
        }
    }

    pub(crate) fn calculate_done_percentage(&self) -> u32 {
        ((self.open_fields as f32 / (self.total_fields - self.mine_count) as f32) * 100_f32) as u32
    }
}

impl Field {
    pub(crate) fn new() -> Self {
        Field {
            value: 0,
            field_state: FieldState::Closed,
            mine: false,
        }
    }
}
