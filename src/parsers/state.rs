use crate::parsers::copy_row::CurrentTableTransforms;
use crate::parsers::types::Column;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct Types {
    types: HashMap<String, HashMap<String, String>>,
}

impl Types {
    pub fn new(initial: HashMap<String, HashMap<String, String>>) -> Self {
        Types { types: initial }
    }

    pub fn insert(&mut self, table_name: &str, thing: HashMap<String, String>) {
        self.types.insert(table_name.to_string(), thing);
    }

    pub fn lookup(&self, table_name: &str, column_name: String) -> Option<String> {
        self.types
            .get(table_name)
            .and_then(|table| table.get(&column_name))
            .map(|column_type| column_type.to_string())
    }
}

pub struct State {
    pub position: Position,
    pub types: Types,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Position {
    Normal,
    InCopy {
        current_table: CurrentTableTransforms,
    },
    InCreateTable {
        table_name: String,
        types: Vec<Column>,
    },
}

impl State {
    pub fn new() -> State {
        State {
            position: Position::Normal,
            types: Types::new(HashMap::new()),
        }
    }

    pub fn update_position(&mut self, new_position: Position) {
        if let (
            Position::InCreateTable {
                table_name,
                types: table_types,
            },
            Position::Normal,
        ) = (self.position.clone(), new_position.clone())
        {
            self.types.insert(
                &table_name,
                table_types
                    .iter()
                    .map(|c| (c.name.clone(), c.data_type.clone()))
                    .collect::<HashMap<String, String>>(),
            );
        }

        self.position = new_position
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsers::types::Column;
    use std::collections::HashMap;

    #[test]
    fn new_creates_default_state() {
        let state = State::new();
        assert_eq!(state.position, Position::Normal);
        assert_eq!(state.types, Types::new(HashMap::new()));
    }

    #[test]
    fn update_position_modifies_position() {
        let mut state = State::new();
        let new_position = Position::InCopy {
            current_table: CurrentTableTransforms {
                table_name: "table-mc-tableface".to_string(),
                transforms: None,
            },
        };

        state.update_position(new_position.clone());
        assert_eq!(state.position, new_position);
    }

    #[test]
    #[allow(non_snake_case)]
    fn if_updating_from_InCreateTable_to_Normal_updates_types() {
        let mut state = State {
            position: Position::InCreateTable {
                table_name: "table-mc-tableface".to_string(),
                types: vec![
                    Column {
                        name: "column".to_string(),
                        data_type: "bigint".to_string(),
                    },
                    Column {
                        name: "column_2".to_string(),
                        data_type: "timestamp with time zone".to_string(),
                    },
                ],
            },
            types: Types::new(HashMap::new()),
        };

        state.update_position(Position::Normal);

        assert_eq!(state.position, Position::Normal);
        assert_eq!(
            state.types,
            Types::new(HashMap::from([(
                "table-mc-tableface".to_string(),
                HashMap::from([
                    ("column".to_string(), "bigint".to_string()),
                    (
                        "column_2".to_string(),
                        "timestamp with time zone".to_string()
                    )
                ])
            )]))
        );
    }
}
