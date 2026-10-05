use crate::krill::{KrillErrors, Table, Value, ValueType};
use std::path::Path;

fn empty_table() -> Table<'static> {
    Table::from_csv(Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/test_data/test input/header_only.csv"
    )))
    .unwrap()
}

fn populated_table() -> Table<'static> {
    let mut table = empty_table();
    table
        .add_row(vec![Value::from(1), Value::from(2.5)], true)
        .unwrap();
    table
        .add_row(vec![Value::from(3), Value::from(4.5)], true)
        .unwrap();
    table
}

#[test]
fn add_row_appends_values_in_column_order() {
    let mut table = populated_table();
    table
        .add_row(vec![Value::from(5), Value::from(6.5)], true)
        .unwrap();
    assert_eq!(table.dimensions(), (2, 3));
    assert_eq!(
        table
            .rows()
            .iter()
            .map(|row| row.values.clone())
            .collect::<Vec<_>>(),
        vec![
            vec![Value::from(1), Value::from(2.5)],
            vec![Value::from(3), Value::from(4.5)],
            vec![Value::from(5), Value::from(6.5)],
        ]
    );
}

#[test]
fn add_row_pads_short_and_empty_rows() {
    let mut table = populated_table();
    table.add_row(vec![Value::from(5)], true).unwrap();
    table.add_row(vec![], true).unwrap();
    assert_eq!(table.dimensions(), (2, 4));
    assert_eq!(table[2].values, vec![Value::from(5), Value::Empty]);
    assert_eq!(table[3].values, vec![Value::Empty, Value::Empty]);
    assert_eq!(table.column_type("float_values").unwrap(), ValueType::Float);
}

#[test]
fn add_row_accepts_explicit_empty_values_in_typed_columns() {
    let mut table = populated_table();
    table
        .add_row(vec![Value::Empty, Value::from(6.5)], true)
        .unwrap();
    assert_eq!(table[2].values, vec![Value::Empty, Value::from(6.5)]);
    assert_eq!(
        table.column_type("integer_values").unwrap(),
        ValueType::Integer
    );
}

#[test]
fn add_row_infers_unset_types_from_first_nonempty_values() {
    let mut table = empty_table();
    table
        .add_row(vec![Value::Empty, Value::Empty], true)
        .unwrap();
    assert_eq!(
        table.column_type("integer_values").unwrap(),
        ValueType::Unset
    );
    assert_eq!(table.column_type("float_values").unwrap(), ValueType::Unset);
    table
        .add_row(vec![Value::from(1), Value::from(2.5)], true)
        .unwrap();
    assert_eq!(
        table.column_type("integer_values").unwrap(),
        ValueType::Integer
    );
    assert_eq!(table.column_type("float_values").unwrap(), ValueType::Float);
    assert_eq!(table[0].values, vec![Value::Empty, Value::Empty]);
}

#[test]
fn add_row_rejects_extra_values_without_changing_table() {
    let mut table = populated_table();
    let rows = table.rows().to_vec();
    let columns = table.column_map.clone();
    let error = table
        .add_row(vec![Value::from(5), Value::from(6.5), Value::from(7)], true)
        .unwrap_err();
    assert!(matches!(
        error.downcast_ref::<KrillErrors>(),
        Some(KrillErrors::TooManyArguments(..))
    ));
    assert_eq!(table.rows(), rows);
    assert_eq!(table.column_map, columns);
}

#[test]
fn add_row_type_error_does_not_partially_infer_columns() {
    let mut table = empty_table();
    table
        .add_column_default(Value::from(2.5), "typed", true)
        .unwrap();
    let rows = table.rows().to_vec();
    let columns = table.column_map.clone();
    let error = table
        .add_row(
            vec![Value::from(1), Value::Empty, Value::from("wrong type")],
            true,
        )
        .unwrap_err();
    assert!(matches!(
        error.downcast_ref::<KrillErrors>(),
        Some(KrillErrors::ColumnTypeMismatched(
            ValueType::Float,
            ValueType::String
        ))
    ));
    assert_eq!(table.rows(), rows);
    assert_eq!(table.column_map, columns);
}

#[test]
fn remove_column_keeps_other_values_and_updates_indices() {
    let mut table = populated_table();
    assert!(table.rem_col("integer_values", true).unwrap().is_none());
    assert_eq!(table.dimensions(), (1, 2));
    assert!(table.column_details("integer_values").is_err());
    assert_eq!(table.column_index("float_values").unwrap(), 0);
    assert_eq!(table.column_type("float_values").unwrap(), ValueType::Float);
    assert_eq!(
        table.column_deep_copy("float_values").unwrap(),
        vec![Value::from(2.5), Value::from(4.5)]
    );
    assert_eq!(table[0].values, vec![Value::from(2.5)]);
    assert_eq!(table[1].values, vec![Value::from(4.5)]);
}

#[test]
fn remove_column_copy_leaves_original_unchanged_and_is_independent() {
    let mut table = populated_table();
    let rows = table.rows().to_vec();
    let columns = table.column_map.clone();
    let mut copy = table.rem_col("integer_values", false).unwrap().unwrap();
    assert_eq!(table.dimensions(), (2, 2));
    assert_eq!(table.rows(), rows);
    assert_eq!(table.column_map, columns);
    assert_eq!(copy.dimensions(), (1, 2));
    assert!(copy.column_details("integer_values").is_err());
    assert_eq!(copy.column_index("float_values").unwrap(), 0);
    assert_eq!(
        copy.column_deep_copy("float_values").unwrap(),
        vec![Value::from(2.5), Value::from(4.5)]
    );
    copy.get_row_mut(0).unwrap().values[0] = Value::from(9.5);
    assert_eq!(table[0].values, vec![Value::from(1), Value::from(2.5)]);
}

#[test]
fn remove_first_middle_or_last_column_preserves_remaining_order() {
    for inplace in [true, false] {
        for (removed, remaining, expected) in [
            (
                "integer_values",
                vec!["float_values", "label"],
                vec![Value::from(2.5), Value::from("kept")],
            ),
            (
                "float_values",
                vec!["integer_values", "label"],
                vec![Value::from(1), Value::from("kept")],
            ),
            (
                "label",
                vec!["integer_values", "float_values"],
                vec![Value::from(1), Value::from(2.5)],
            ),
        ] {
            let mut table = populated_table();
            table
                .add_column_default(Value::from("kept"), "label", true)
                .unwrap();
            let copy = table.rem_col(removed, inplace).unwrap();
            let result = copy.as_ref().unwrap_or(&table);
            assert_eq!(result.dimensions(), (2, 2));
            assert_eq!(result[0].values, expected);
            assert!(result.column_details(removed).is_err());
            for (index, name) in remaining.iter().enumerate() {
                assert_eq!(result.column_index(name).unwrap(), index);
            }
        }
    }
}

#[test]
fn remove_columns_repeatedly_keeps_row_count_even_with_no_columns() {
    let mut table = populated_table();
    table.rem_col("integer_values", true).unwrap();
    table.rem_col("float_values", true).unwrap();
    assert_eq!(table.dimensions(), (0, 2));
    assert!(table.column_map.is_empty());
    assert!(table.rows().iter().all(|row| row.values.is_empty()));
}

#[test]
fn remove_column_from_table_without_rows_updates_schema() {
    let mut table = empty_table();
    table.rem_col("integer_values", true).unwrap();
    assert_eq!(table.dimensions(), (1, 0));
    assert_eq!(table.column_index("float_values").unwrap(), 0);
    assert!(table.column_details("integer_values").is_err());
}

#[test]
fn remove_missing_column_returns_error_without_changing_table() {
    for inplace in [true, false] {
        let mut table = populated_table();
        let rows = table.rows().to_vec();
        let columns = table.column_map.clone();
        let error = table.rem_col("missing", inplace).unwrap_err();
        assert!(matches!(error.downcast_ref::<KrillErrors>(),
            Some(KrillErrors::ColumnNotFound(name)) if name == "missing"));
        assert_eq!(table.dimensions(), (2, 2));
        assert_eq!(table.rows(), rows);
        assert_eq!(table.column_map, columns);
    }
}

#[test]
fn add_columns_respects_inplace_and_fills_every_row() {
    for inplace in [true, false] {
        for default in [Value::Empty, Value::from("new")] {
            let mut table = populated_table();
            let rows = table.rows().to_vec();
            let columns = table.column_map.clone();
            let copy = if default.is_empty() {
                table.add_column_empty("added", inplace).unwrap()
            } else {
                table
                    .add_column_default(default.clone(), "added", inplace)
                    .unwrap()
            };
            assert_eq!(copy.is_none(), inplace);
            let result = copy.as_ref().unwrap_or(&table);
            assert_eq!(result.dimensions(), (3, 2));
            assert_eq!(result.column_index("added").unwrap(), 2);
            assert_eq!(
                result.column_type("added").unwrap(),
                if default.is_empty() {
                    ValueType::Unset
                } else {
                    ValueType::String
                }
            );
            assert_eq!(
                result[0].values,
                vec![Value::from(1), Value::from(2.5), default.clone()]
            );
            assert_eq!(
                result[1].values,
                vec![Value::from(3), Value::from(4.5), default]
            );
            if !inplace {
                assert_eq!(table.dimensions(), (2, 2));
                assert_eq!(table.rows(), rows);
                assert_eq!(table.column_map, columns);
            }
        }
    }
}

#[test]
fn adding_columns_to_table_without_rows_creates_usable_schema() {
    let mut table = empty_table();
    table.add_column_empty("optional", true).unwrap();
    table
        .add_column_default(Value::from("default"), "label", true)
        .unwrap();
    assert_eq!(table.dimensions(), (4, 0));
    assert_eq!(table.column_type("optional").unwrap(), ValueType::Unset);
    assert_eq!(table.column_type("label").unwrap(), ValueType::String);
    table
        .add_row(
            vec![
                Value::from(1),
                Value::from(2.5),
                Value::Empty,
                Value::from("actual"),
            ],
            true,
        )
        .unwrap();
    assert_eq!(
        table[0].values,
        vec![
            Value::from(1),
            Value::from(2.5),
            Value::Empty,
            Value::from("actual")
        ]
    );
}

#[test]
fn duplicate_column_additions_preserve_existing_schema_and_values_in_both_modes() {
    for inplace in [true, false] {
        for empty in [true, false] {
            let mut table = populated_table();
            let columns = table.column_map.clone();
            let rows = table.rows().to_vec();
            let copy = if empty {
                table.add_column_empty("integer_values", inplace).unwrap()
            } else {
                table
                    .add_column_default(Value::from("different type"), "integer_values", inplace)
                    .unwrap()
            };
            assert_eq!(copy.is_none(), inplace);
            let result = copy.as_ref().unwrap_or(&table);
            assert_eq!(result.dimensions(), (2, 2));
            assert_eq!(result.column_map, columns);
            assert_eq!(result.rows(), rows);
        }
    }
}

#[test]
fn fillna_copy_fills_all_rows_and_preserves_original() {
    let mut table = empty_table();
    for _ in 0..4 {
        table.add_row(vec![], true).unwrap();
    }
    let copy = table
        .fillna_column("integer_values", |_, _| Ok(Value::from(7)), false)
        .unwrap()
        .unwrap();
    assert_eq!(
        copy.column_deep_copy("integer_values").unwrap(),
        vec![Value::from(7); 4]
    );
    assert_eq!(
        copy.column_deep_copy("float_values").unwrap(),
        vec![Value::Empty; 4]
    );
    assert_eq!(
        table.column_deep_copy("integer_values").unwrap(),
        vec![Value::Empty; 4]
    );
}

#[test]
fn add_row_respects_inplace_and_copy_modes() {
    for inplace in [true, false] {
        let mut table = populated_table();
        let rows = table.rows().to_vec();
        let copy = table
            .add_row(vec![Value::from(5), Value::from(6.5)], inplace)
            .unwrap();
        assert_eq!(copy.is_none(), inplace);
        let result = copy.as_ref().unwrap_or(&table);
        assert_eq!(result.dimensions(), (2, 3));
        assert_eq!(&result.rows()[..2], rows);
        assert_eq!(result[2].values, vec![Value::from(5), Value::from(6.5)]);
        if !inplace {
            assert_eq!(table.rows(), rows);
        }
    }
}

#[test]
fn add_row_copy_preserves_original_unset_types() {
    let mut table = empty_table();
    let copy = table
        .add_row(vec![Value::from(1), Value::from(2.5)], false)
        .unwrap()
        .unwrap();
    assert_eq!(table.dimensions(), (2, 0));
    assert_eq!(
        table.column_type("integer_values").unwrap(),
        ValueType::Unset
    );
    assert_eq!(
        copy.column_type("integer_values").unwrap(),
        ValueType::Integer
    );
    assert_eq!(copy[0].values, vec![Value::from(1), Value::from(2.5)]);
}

#[test]
fn remove_row_respects_inplace_and_preserves_remaining_order() {
    for inplace in [true, false] {
        for index in 0..3 {
            let mut table = populated_table();
            table
                .add_row(vec![Value::from(5), Value::from(6.5)], true)
                .unwrap();
            let original = table.rows().to_vec();
            let columns = table.column_map.clone();
            let copy = table.remove_row(index, inplace).unwrap();
            assert_eq!(copy.is_none(), inplace);
            let result = copy.as_ref().unwrap_or(&table);
            let expected = match index {
                0 => vec![
                    vec![Value::from(3), Value::from(4.5)],
                    vec![Value::from(5), Value::from(6.5)],
                ],
                1 => vec![
                    vec![Value::from(1), Value::from(2.5)],
                    vec![Value::from(5), Value::from(6.5)],
                ],
                _ => vec![
                    vec![Value::from(1), Value::from(2.5)],
                    vec![Value::from(3), Value::from(4.5)],
                ],
            };
            assert_eq!(result.dimensions(), (2, 2));
            assert_eq!(
                result
                    .rows()
                    .iter()
                    .map(|row| row.values.clone())
                    .collect::<Vec<_>>(),
                expected
            );
            assert_eq!(result.column_map, columns);
            if !inplace {
                assert_eq!(table.rows(), original);
            }
        }
    }
}

#[test]
fn removing_only_row_preserves_columns_in_both_modes() {
    for inplace in [true, false] {
        let mut table = empty_table();
        table
            .add_row(vec![Value::from(1), Value::from(2.5)], true)
            .unwrap();
        let columns = table.column_map.clone();
        let copy = table.remove_row(0, inplace).unwrap();
        let result = copy.as_ref().unwrap_or(&table);
        assert_eq!(result.dimensions(), (2, 0));
        assert_eq!(result.column_map, columns);
        if !inplace {
            assert_eq!(table.dimensions(), (2, 1));
        }
    }
}

#[test]
fn removing_invalid_row_preserves_original_in_both_modes() {
    for inplace in [true, false] {
        let mut table = populated_table();
        let rows = table.rows().to_vec();
        let error = table.remove_row(2, inplace).unwrap_err();
        assert!(matches!(
            error.downcast_ref::<KrillErrors>(),
            Some(KrillErrors::RowOutOfBounds(2, 2))
        ));
        assert_eq!(table.rows(), rows);
    }
}

#[test]
fn replacing_value_respects_inplace_and_preserves_other_values() {
    for inplace in [true, false] {
        let mut table = populated_table();
        let rows = table.rows().to_vec();
        let copy = table
            .replace_value_on_row(0, 1, Value::from(9.5), inplace)
            .unwrap();
        assert_eq!(copy.is_none(), inplace);
        let result = copy.as_ref().unwrap_or(&table);
        assert_eq!(result.dimensions(), (2, 2));
        assert_eq!(result[0].values, vec![Value::from(1), Value::from(9.5)]);
        assert_eq!(result[1].values, vec![Value::from(3), Value::from(4.5)]);
        if !inplace {
            assert_eq!(table.rows(), rows);
        }
    }
}
