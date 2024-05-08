use crate::parsers::strategy_errors::{DbErrors, ValidationErrors};
use crate::parsers::strategy_structs::*;
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Debug, PartialEq, Eq)]
pub struct Strategies {
    tables: HashMap<String, TableStrategy>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TableStrategy {
    Columns(HashMap<String, ColumnInfo>),
    Truncate,
}

impl TableStrategy {
    fn to_columns(self) -> HashMap<String, ColumnInfo> {
        if let TableStrategy::Columns(c) = self {
            c
        } else {
            panic!("Not columns!")
        }
    }
}

impl Strategies {
    pub fn new() -> Strategies {
        Strategies {
            tables: HashMap::new(),
        }
    }

    pub fn from_strategies_in_file(
        strategies_in_file: Vec<StrategyInFile>,
        transformer_overrides: &TransformerOverrides,
    ) -> Result<Strategies, ValidationErrors> {
        let mut transformed_strategies = Strategies::new();
        let mut errors = ValidationErrors::new();

        for strategy in strategies_in_file {
            let mut columns = HashMap::<String, ColumnInfo>::new();
            for column in strategy.columns {
                if (column.data_category == DataCategory::PotentialPii
                    || column.data_category == DataCategory::Pii)
                    && column.transformer.name == TransformerType::Identity
                {
                    errors
                        .unanonymised_pii
                        .push(create_simple_column(&column.name, &strategy.table_name));
                }
                if column.data_category == DataCategory::Unknown {
                    errors
                        .unknown_data_categories
                        .push(create_simple_column(&column.name, &strategy.table_name));
                }
                if column.transformer.name == TransformerType::Error {
                    errors
                        .error_transformer_types
                        .push(create_simple_column(&column.name, &strategy.table_name));
                }
                let result = columns.insert(
                    column.name.clone(),
                    ColumnInfo {
                        data_category: column.data_category.clone(),
                        name: column.name.clone(),
                        transformer: transformer(column, transformer_overrides),
                    },
                );
                if let Some(dupe) = result {
                    errors.duplicate_columns.push(create_simple_column(
                        &dupe.name,
                        &strategy.table_name.clone(),
                    ))
                }
            }

            let result = transformed_strategies.insert(strategy.table_name.clone(), columns);
            if result.is_some() {
                errors.duplicate_tables.push(strategy.table_name);
            }
        }

        if ValidationErrors::is_empty(&errors) {
            Ok(transformed_strategies)
        } else {
            //TODO sort/order errors somehow or maybe only do that when we log them out??
            Err(errors)
        }
    }

    pub fn for_table(&self, table_name: &str) -> Option<&TableStrategy> {
        self.tables.get(table_name)
    }

    pub fn insert(
        &mut self,
        table_name: String,
        columns: HashMap<String, ColumnInfo>,
    ) -> Option<TableStrategy> {
        self.tables
            .insert(table_name, TableStrategy::Columns(columns))
    }

    // TODO here, we need to work out how to do validation for tuncation
    pub fn validate_against_db(
        &self,
        columns_from_db: HashSet<SimpleColumn>,
    ) -> Result<(), DbErrors> {
        let (columns, truncate): (
            HashMap<String, TableStrategy>,
            HashMap<String, TableStrategy>,
        ) = self
            .tables
            .into_iter()
            .partition(|(table, table_strategy)| match table_strategy {
                TableStrategy::Columns(columns) => true,
                TableStrategy::Truncate => false,
            });

        let columns_from_strategy_file: HashSet<SimpleColumn> = columns
            .iter()
            .flat_map(|(table, columns)| {
                return columns
                    .to_columns()
                    .iter()
                    .map(|(column, _)| create_simple_column(column, table));
            })
            .collect();

        let mut errors = DbErrors {
            missing_from_strategy_file: columns_from_db
                .difference(&columns_from_strategy_file)
                .cloned()
                .collect(),
            missing_from_db: columns_from_strategy_file
                .difference(&columns_from_db)
                .cloned()
                .collect(),
        };

        if DbErrors::is_empty(&errors) {
            Ok(())
        } else {
            // TODO i wanted to do like errors.sort() and errors.is_empty()
            // above but couldnt work out the ownership :(
            errors.missing_from_strategy_file.sort();
            errors.missing_from_db.sort();
            Err(errors)
        }
    }

    #[allow(dead_code)] //This is used in tests for convenience
    pub fn transformer_for_column<'a>(
        &self,
        table_name: &'a str,
        column_name: &'a str,
    ) -> Option<Transformer> {
        self.tables
            .get(table_name)
            .and_then(|table| match table {
                TableStrategy::Columns(columns) => columns.get(column_name),
                TableStrategy::Truncate => None,
            })
            .map(|column| column.transformer.clone())
    }

    #[allow(dead_code)] //This is used in tests for convenience
    pub fn new_from(table_name: String, columns: HashMap<String, ColumnInfo>) -> Strategies {
        Strategies {
            tables: HashMap::from([(table_name, TableStrategy::Columns(columns))]),
        }
    }
}

fn create_simple_column(column_name: &str, table_name: &str) -> SimpleColumn {
    SimpleColumn {
        table_name: table_name.to_string(),
        column_name: column_name.to_string(),
    }
}

fn apply_transformer_overrides(
    data_category: DataCategory,
    overrides: &TransformerOverrides,
    transformer: Transformer,
) -> Transformer {
    match data_category {
        DataCategory::PotentialPii if overrides.allow_potential_pii => Transformer {
            name: TransformerType::Identity,
            args: None,
        },
        DataCategory::CommerciallySensitive if overrides.allow_commercially_sensitive => {
            Transformer {
                name: TransformerType::Identity,
                args: None,
            }
        }
        _ if overrides.scramble_blank && transformer.name == TransformerType::Scramble => {
            Transformer {
                name: TransformerType::ScrambleBlank,
                args: None,
            }
        }
        _ => transformer,
    }
}

fn transformer(column: ColumnInFile, overrides: &TransformerOverrides) -> Transformer {
    apply_transformer_overrides(column.data_category, overrides, column.transformer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsers::strategy_structs::{ColumnInfo, TransformerType};
    use std::collections::HashMap;

    #[test]
    fn validate_against_db_returns_ok_with_matching_fields() {
        let mut strategies =
            create_strategy("public.person", [create_column("first_name")].into_iter());

        add_table(
            &mut strategies,
            "public.location",
            [create_column("postcode")].into_iter(),
        );

        let columns_from_db = HashSet::from([
            create_simple_column("public.person", "first_name"),
            create_simple_column("public.location", "postcode"),
        ]);

        let result = strategies.validate_against_db(columns_from_db);

        assert!(result.is_ok());
    }

    #[test]
    fn validate_against_db_returns_fields_missing_from_strategy_file_that_are_in_the_db() {
        let strategies =
            create_strategy("public.person", [create_column("first_name")].into_iter());

        let columns_from_db = HashSet::from([
            create_simple_column("public.person", "first_name"),
            create_simple_column("public.location", "postcode"),
        ]);

        let result = strategies.validate_against_db(columns_from_db);

        let error = result.unwrap_err();
        assert!(error.missing_from_db.is_empty());
        assert_eq!(
            error.missing_from_strategy_file,
            vec!(create_simple_column("public.location", "postcode"))
        );
    }

    #[test]
    fn validate_against_db_returns_fields_missing_from_the_db_but_are_in_the_strategy_file() {
        let mut strategies =
            create_strategy("public.person", [create_column("first_name")].into_iter());

        add_table(
            &mut strategies,
            "public.location",
            [create_column("postcode")].into_iter(),
        );

        let columns_from_db = HashSet::from([create_simple_column("public.person", "first_name")]);

        let result = strategies.validate_against_db(columns_from_db);

        let error = result.unwrap_err();
        assert!(error.missing_from_strategy_file.is_empty());
        assert_eq!(
            error.missing_from_db,
            vec!(create_simple_column("public.location", "postcode"))
        );
    }

    #[test]
    fn validate_against_db_returns_fields_missing_both() {
        let strategies =
            create_strategy("public.person", [create_column("first_name")].into_iter());

        let columns_from_db = HashSet::from([create_simple_column("public.location", "postcode")]);

        let result = strategies.validate_against_db(columns_from_db);

        let error = result.unwrap_err();
        assert_eq!(
            error.missing_from_strategy_file,
            vec!(create_simple_column("public.location", "postcode"))
        );
        assert_eq!(
            error.missing_from_db,
            vec!(create_simple_column("public.person", "first_name"))
        );
    }

    const TABLE_NAME: &str = "gert_lush_table";
    const PII_COLUMN_NAME: &str = "pii_column";
    const COMMERCIALLY_SENSITIVE_COLUMN_NAME: &str = "commercially_sensitive_column";
    const SCRAMBLED_COLUMN_NAME: &str = "scrambled_column";

    #[test]
    fn from_strategies_in_file_can_parse_file_contents_into_hashmaps() {
        let column_name = "column1";

        let strategies = vec![StrategyInFile {
            table_name: TABLE_NAME.to_string(),
            description: "description".to_string(),
            truncate: false,
            columns: vec![column_in_file(
                DataCategory::Pii,
                column_name,
                TransformerType::Scramble,
            )],
        }];

        let expected = Strategies::new_from(
            TABLE_NAME.to_string(),
            HashMap::from([(
                column_name.to_string(),
                ColumnInfo::builder()
                    .with_name(column_name)
                    .with_data_category(DataCategory::Pii)
                    .with_transformer(TransformerType::Scramble, None)
                    .build(),
            )]),
        );
        let parsed = Strategies::from_strategies_in_file(strategies, &TransformerOverrides::none())
            .expect("we shouldnt have duplicate columns!");
        assert_eq!(expected, parsed);
    }

    #[test]
    fn from_strategies_in_file_returns_errors_for_duplicate_table_and_column_definitions() {
        let table2_name = "daps";
        let column_name = "column1";
        let duplicated_column =
            column_in_file(DataCategory::Pii, column_name, TransformerType::Scramble);

        let strategies = vec![
            StrategyInFile {
                table_name: TABLE_NAME.to_string(),
                description: "description".to_string(),
                truncate: false,
                columns: vec![],
            },
            StrategyInFile {
                table_name: TABLE_NAME.to_string(),
                description: "description".to_string(),
                truncate: false,
                columns: vec![],
            },
            StrategyInFile {
                table_name: table2_name.to_string(),
                description: "description".to_string(),
                truncate: false,
                columns: vec![duplicated_column.clone(), duplicated_column],
            },
        ];

        let error = Strategies::from_strategies_in_file(strategies, &TransformerOverrides::none())
            .expect_err("We should have a duplicate table error");

        assert_eq!(error.duplicate_tables, vec![TABLE_NAME.to_string()]);
        assert_eq!(
            error.duplicate_columns,
            vec![create_simple_column(table2_name, column_name)]
        );
    }

    #[test]
    fn from_strategies_in_file_returns_errors_for_columns_missing_data_category() {
        let strategies = vec![StrategyInFile {
            table_name: "public.person".to_string(),
            description: "description".to_string(),
            truncate: false,
            columns: vec![column_in_file(
                DataCategory::Unknown,
                "first_name",
                TransformerType::Identity,
            )],
        }];

        let result = Strategies::from_strategies_in_file(strategies, &TransformerOverrides::none());

        let error = result.unwrap_err();
        assert_eq!(
            error.unknown_data_categories,
            vec!(create_simple_column("public.person", "first_name"))
        );
    }

    #[test]
    fn from_strategies_in_file_returns_errors_for_columns_with_error_transformer_types() {
        let strategies = vec![StrategyInFile {
            table_name: "public.person".to_string(),
            description: "description".to_string(),
            truncate: false,
            columns: vec![column_in_file(
                DataCategory::General,
                "first_name",
                TransformerType::Error,
            )],
        }];

        let result = Strategies::from_strategies_in_file(strategies, &TransformerOverrides::none());

        let error = result.unwrap_err();
        assert_eq!(
            error.error_transformer_types,
            vec!(create_simple_column("public.person", "first_name"))
        );
    }

    #[test]
    fn from_strategies_in_file_returns_errors_for_pii_columns_with_identity_transformer() {
        let strategies = vec![StrategyInFile {
            table_name: "public.person".to_string(),
            description: "description".to_string(),
            truncate: false,
            columns: vec![
                column_in_file(DataCategory::Pii, "first_name", TransformerType::Identity),
                column_in_file(
                    DataCategory::PotentialPii,
                    "last_name",
                    TransformerType::Identity,
                ),
            ],
        }];

        let result = Strategies::from_strategies_in_file(strategies, &TransformerOverrides::none());

        let error = result.unwrap_err();

        assert_eq!(
            error.unanonymised_pii,
            vec!(
                create_simple_column("public.person", "first_name"),
                create_simple_column("public.person", "last_name")
            )
        );
    }

    #[test]
    fn from_strategies_in_file_ignores_transformers_for_potential_pii_if_flag_provided() {
        let strategies = vec![StrategyInFile {
            table_name: TABLE_NAME.to_string(),
            description: "description".to_string(),
            truncate: false,
            columns: vec![
                column_in_file(
                    DataCategory::PotentialPii,
                    PII_COLUMN_NAME,
                    TransformerType::Scramble,
                ),
                column_in_file(
                    DataCategory::CommerciallySensitive,
                    COMMERCIALLY_SENSITIVE_COLUMN_NAME,
                    TransformerType::Scramble,
                ),
            ],
        }];

        let parsed = Strategies::from_strategies_in_file(
            strategies,
            &TransformerOverrides {
                allow_potential_pii: true,
                allow_commercially_sensitive: false,
                ..Default::default()
            },
        )
        .expect("we shouldnt have duplicate columns!");
        let pii_column_transformer = transformer_for_column(PII_COLUMN_NAME, &parsed);
        let commercially_sensitive_transformer =
            transformer_for_column(COMMERCIALLY_SENSITIVE_COLUMN_NAME, &parsed);

        assert_eq!(pii_column_transformer.name, TransformerType::Identity);
        assert_eq!(pii_column_transformer.args, None);

        assert_eq!(
            commercially_sensitive_transformer.name,
            TransformerType::Scramble
        );
        assert_eq!(commercially_sensitive_transformer.args, None);
    }

    #[test]
    fn from_strategies_in_file_ignores_transformers_for_commercially_sensitive_if_flag_provided() {
        let strategies = vec![StrategyInFile {
            table_name: TABLE_NAME.to_string(),
            description: "description".to_string(),
            truncate: false,
            columns: vec![
                column_in_file(
                    DataCategory::PotentialPii,
                    PII_COLUMN_NAME,
                    TransformerType::Scramble,
                ),
                column_in_file(
                    DataCategory::CommerciallySensitive,
                    COMMERCIALLY_SENSITIVE_COLUMN_NAME,
                    TransformerType::Scramble,
                ),
            ],
        }];

        let parsed = Strategies::from_strategies_in_file(
            strategies,
            &TransformerOverrides {
                allow_potential_pii: false,
                allow_commercially_sensitive: true,
                ..Default::default()
            },
        )
        .expect("we shouldnt have duplicate columns!");

        let commercially_sensitive_transformer =
            transformer_for_column(COMMERCIALLY_SENSITIVE_COLUMN_NAME, &parsed);
        let pii_column_transformer = transformer_for_column(PII_COLUMN_NAME, &parsed);

        assert_eq!(
            commercially_sensitive_transformer.name,
            TransformerType::Identity
        );
        assert_eq!(commercially_sensitive_transformer.args, None);

        assert_eq!(pii_column_transformer.name, TransformerType::Scramble);
        assert_eq!(pii_column_transformer.args, None);
    }

    #[test]
    fn from_strategies_in_file_modifies_transformer_for_scramble_if_flag_provided() {
        let strategies = vec![StrategyInFile {
            table_name: TABLE_NAME.to_string(),
            description: "description".to_string(),
            truncate: false,
            columns: vec![column_in_file(
                DataCategory::General,
                SCRAMBLED_COLUMN_NAME,
                TransformerType::Scramble,
            )],
        }];

        let parsed = Strategies::from_strategies_in_file(
            strategies,
            &TransformerOverrides {
                scramble_blank: true,
                ..Default::default()
            },
        )
        .expect("we shouldnt have duplicate columns!");

        let scramble_transformer = transformer_for_column(SCRAMBLED_COLUMN_NAME, &parsed);

        assert_eq!(scramble_transformer.name, TransformerType::ScrambleBlank);
    }

    #[test]
    fn from_strategies_in_file_can_combine_override_flags() {
        let strategies = vec![StrategyInFile {
            table_name: TABLE_NAME.to_string(),
            description: "description".to_string(),
            truncate: false,
            columns: vec![
                column_in_file(
                    DataCategory::PotentialPii,
                    PII_COLUMN_NAME,
                    TransformerType::Scramble,
                ),
                column_in_file(
                    DataCategory::CommerciallySensitive,
                    COMMERCIALLY_SENSITIVE_COLUMN_NAME,
                    TransformerType::Scramble,
                ),
            ],
        }];

        let parsed = Strategies::from_strategies_in_file(
            strategies,
            &TransformerOverrides {
                allow_potential_pii: true,
                allow_commercially_sensitive: true,
                scramble_blank: true,
            },
        )
        .expect("we shouldnt have duplicate columns!");

        // Both of these override scramble_blank

        let commercially_sensitive_transformer =
            transformer_for_column(COMMERCIALLY_SENSITIVE_COLUMN_NAME, &parsed);
        let pii_column_transformer = transformer_for_column(PII_COLUMN_NAME, &parsed);

        assert_eq!(
            commercially_sensitive_transformer.name,
            TransformerType::Identity
        );
        assert_eq!(pii_column_transformer.name, TransformerType::Identity);
    }

    fn transformer_for_column(column_name: &str, strategies: &Strategies) -> Transformer {
        strategies
            .transformer_for_column(TABLE_NAME, column_name)
            .expect("expecting a transformer!")
    }

    fn column_in_file(
        data_category: DataCategory,
        name: &str,
        transformer_type: TransformerType,
    ) -> ColumnInFile {
        ColumnInFile {
            data_category,
            description: name.to_string(),
            name: name.to_string(),
            transformer: Transformer {
                name: transformer_type,
                args: None,
            },
        }
    }

    fn create_strategy<I>(table_name: &str, columns: I) -> Strategies
    where
        I: Iterator<Item = (String, ColumnInfo)>,
    {
        let mut strategies = Strategies::new();
        strategies.insert(table_name.to_string(), HashMap::from_iter(columns));
        strategies
    }

    fn add_table<I>(strategies: &mut Strategies, table_name: &str, columns: I)
    where
        I: Iterator<Item = (String, ColumnInfo)>,
    {
        strategies.insert(table_name.to_string(), HashMap::from_iter(columns));
    }

    fn create_column(column_name: &str) -> (String, ColumnInfo) {
        create_column_with_data_and_transformer_type(
            column_name,
            DataCategory::General,
            TransformerType::Identity,
        )
    }

    fn create_column_with_data_and_transformer_type(
        column_name: &str,
        data_category: DataCategory,
        transformer_type: TransformerType,
    ) -> (String, ColumnInfo) {
        (
            column_name.to_string(),
            ColumnInfo::builder()
                .with_name(column_name)
                .with_data_category(data_category)
                .with_transformer(transformer_type, None)
                .build(),
        )
    }

    fn create_simple_column(table_name: &str, column_name: &str) -> SimpleColumn {
        SimpleColumn {
            table_name: table_name.to_string(),
            column_name: column_name.to_string(),
        }
    }
}
