use crate::krill::{FilterType, Row, Table, Value};
use std::path::Path;

fn table_with_rows(rows: Vec<Vec<Value>>) -> Table<'static> {
    let mut table = Table::from_csv(Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/test_data/test input/header_only.csv"
    )))
    .unwrap();
    for row in rows {
        table.add_row(row, true).unwrap();
    }
    table
}

fn mixed_rows() -> Vec<Vec<Value>> {
    vec![
        vec![Value::Empty, Value::Empty],
        vec![Value::Empty, Value::Empty],
        vec![Value::from(1), Value::from(2.5)],
        vec![Value::Empty, Value::from(3.5)],
        vec![Value::from(4), Value::Empty],
        vec![Value::Empty, Value::Empty],
    ]
}

// These tests check shared contracts without assuming whether matches are kept or removed.
#[test]
fn filtering_a_table_without_rows_succeeds_inplace() {
    let mut table = table_with_rows(vec![]);
    assert!(
        table
            .filter_row(FilterType::<fn(&Row) -> bool>::Empty, true)
            .unwrap()
            .is_none()
    );
    assert_eq!(table.dimensions(), (2, 0));
}

#[test]
fn filtering_a_table_without_rows_returns_a_copy() {
    let mut table = table_with_rows(vec![]);
    let copy = table
        .filter_row(FilterType::<fn(&Row) -> bool>::NonEmpty, false)
        .unwrap()
        .unwrap();
    assert_eq!(copy.dimensions(), (2, 0));
    assert_eq!(copy.column_map, table.column_map);
    assert_eq!(table.dimensions(), (2, 0));
}

#[test]
fn empty_filter_copy_preserves_original_rows_and_schema() {
    let rows = mixed_rows();
    let mut table = table_with_rows(rows.clone());
    let columns = table.column_map.clone();
    let _copy = table
        .filter_row(FilterType::<fn(&Row) -> bool>::Empty, false)
        .unwrap()
        .unwrap();
    assert_eq!(
        table
            .rows()
            .iter()
            .map(|row| row.values.clone())
            .collect::<Vec<_>>(),
        rows
    );
    assert_eq!(table.column_map, columns);
    assert_eq!(table.dimensions(), (2, 6));
}

#[test]
fn conditional_filter_copy_preserves_original_rows_and_schema() {
    let rows = mixed_rows();
    let mut table = table_with_rows(rows.clone());
    let columns = table.column_map.clone();
    let _copy = table
        .filter_row(
            FilterType::Conditional(|row: &Row| row.values[0].is_empty()),
            false,
        )
        .unwrap()
        .unwrap();
    assert_eq!(
        table
            .rows()
            .iter()
            .map(|row| row.values.clone())
            .collect::<Vec<_>>(),
        rows
    );
    assert_eq!(table.column_map, columns);
}

#[test]
fn conditional_filter_with_all_matches_succeeds_inplace() {
    let mut table = table_with_rows(mixed_rows());
    let columns = table.column_map.clone();
    assert!(
        table
            .filter_row(FilterType::Conditional(|_: &Row| true), true)
            .unwrap()
            .is_none()
    );
    assert_eq!(table.column_map, columns);
}

#[test]
fn empty_filter_with_adjacent_matches_succeeds_inplace() {
    let mut table = table_with_rows(mixed_rows());
    let columns = table.column_map.clone();
    assert!(
        table
            .filter_row(FilterType::<fn(&Row) -> bool>::Empty, true)
            .unwrap()
            .is_none()
    );
    assert_eq!(table.column_map, columns);
}

fn assert_rows(table: &Table, expected: &[Vec<Value>]) {
    assert_eq!(table.dimensions(), (2, expected.len()));
    assert_eq!(
        table
            .rows()
            .iter()
            .map(|row| row.values.clone())
            .collect::<Vec<_>>(),
        expected
    );
}

macro_rules! filter_case {
    ($name:ident, $inplace:expr, $filter:expr, $input:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let input = $input;
            let mut table = table_with_rows(input.clone());
            let columns = table.column_map.clone();
            let outcome = table.filter_row($filter, $inplace);
            let actual = match &outcome {
                Ok(Some(copy)) => copy.rows(),
                _ => table.rows(),
            };
            eprintln!("{}: expected remaining {:?}; actual remaining {:?}; error {:?}",
                stringify!($name), $expected,
                actual.iter().map(|row| &row.values).collect::<Vec<_>>(),
                outcome.as_ref().err());
            let copy = outcome.unwrap();
            assert_eq!(copy.is_none(), $inplace);
            let result = copy.as_ref().unwrap_or(&table);
            assert_rows(result, &$expected);
            assert_eq!(result.column_map, columns);
            if !$inplace {
                assert_rows(&table, &input);
                assert_eq!(table.column_map, columns);
            }
        }
    };
}

filter_case!(
    empty_filter_removes_only_fully_empty_rows_inplace,
    true,
    FilterType::<fn(&Row) -> bool>::Empty,
    mixed_rows(),
    vec![
        vec![Value::from(1), Value::from(2.5)],
        vec![Value::Empty, Value::from(3.5)],
        vec![Value::from(4), Value::Empty]
    ]
);
filter_case!(
    empty_filter_removes_only_fully_empty_rows_in_copy,
    false,
    FilterType::<fn(&Row) -> bool>::Empty,
    mixed_rows(),
    vec![
        vec![Value::from(1), Value::from(2.5)],
        vec![Value::Empty, Value::from(3.5)],
        vec![Value::from(4), Value::Empty]
    ]
);
filter_case!(
    nonempty_filter_removes_partially_and_fully_populated_rows_inplace,
    true,
    FilterType::<fn(&Row) -> bool>::NonEmpty,
    mixed_rows(),
    vec![vec![Value::Empty, Value::Empty]; 3]
);
filter_case!(
    nonempty_filter_removes_partially_and_fully_populated_rows_in_copy,
    false,
    FilterType::<fn(&Row) -> bool>::NonEmpty,
    mixed_rows(),
    vec![vec![Value::Empty, Value::Empty]; 3]
);
filter_case!(
    conditional_filter_removes_matching_rows_and_preserves_order_inplace,
    true,
    FilterType::Conditional(|row: &Row| !row.values[0].is_empty()),
    mixed_rows(),
    vec![
        vec![Value::Empty, Value::Empty],
        vec![Value::Empty, Value::Empty],
        vec![Value::Empty, Value::from(3.5)],
        vec![Value::Empty, Value::Empty]
    ]
);
filter_case!(
    conditional_filter_removes_matching_rows_and_preserves_order_in_copy,
    false,
    FilterType::Conditional(|row: &Row| !row.values[0].is_empty()),
    mixed_rows(),
    vec![
        vec![Value::Empty, Value::Empty],
        vec![Value::Empty, Value::Empty],
        vec![Value::Empty, Value::from(3.5)],
        vec![Value::Empty, Value::Empty]
    ]
);
filter_case!(
    conditional_filter_with_no_matches_preserves_all_rows_inplace,
    true,
    FilterType::Conditional(|_: &Row| false),
    mixed_rows(),
    mixed_rows()
);
filter_case!(
    conditional_filter_with_no_matches_preserves_all_rows_in_copy,
    false,
    FilterType::Conditional(|_: &Row| false),
    mixed_rows(),
    mixed_rows()
);
filter_case!(
    conditional_filter_removes_all_rows_inplace,
    true,
    FilterType::Conditional(|_: &Row| true),
    mixed_rows(),
    Vec::<Vec<Value>>::new()
);
filter_case!(
    conditional_filter_removes_all_rows_in_copy,
    false,
    FilterType::Conditional(|_: &Row| true),
    mixed_rows(),
    Vec::<Vec<Value>>::new()
);
filter_case!(
    empty_filter_removes_the_only_row_inplace,
    true,
    FilterType::<fn(&Row) -> bool>::Empty,
    vec![vec![Value::Empty, Value::Empty]],
    Vec::<Vec<Value>>::new()
);
filter_case!(
    empty_filter_removes_the_only_row_in_copy,
    false,
    FilterType::<fn(&Row) -> bool>::Empty,
    vec![vec![Value::Empty, Value::Empty]],
    Vec::<Vec<Value>>::new()
);
