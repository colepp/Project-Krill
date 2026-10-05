use crate::krill::{FilterType, FindRowResultTypes, Row, Table, Value};
use std::path::Path;

fn sample() -> Table<'static> {
    let mut table = empty_table();
    for values in [
        vec![Value::Empty, Value::Empty],
        vec![Value::from(1), Value::from(2.5)],
        vec![Value::Empty, Value::from(3.5)],
        vec![Value::from(4), Value::Empty],
        vec![Value::Empty, Value::Empty],
    ] {
        table.add_row(values, true).unwrap();
    }
    table
}

fn empty_table() -> Table<'static> {
    Table::from_csv(Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/test_data/test input/header_only.csv"
    )))
    .unwrap()
}

fn assert_immutable_matches<T: Fn(&Row) -> bool>(filter: FilterType<T>, expected: &[usize]) {
    let table = sample();
    let original = table.rows().to_vec();
    let columns = table.column_map.clone();
    let matches = table.find_rows(filter).unwrap();
    assert_eq!(matches.len(), expected.len());
    for (result, &expected_index) in matches.iter().zip(expected) {
        match result {
            FindRowResultTypes::FindRowResult(row, index) => {
                assert_eq!(*index, expected_index);
                assert_eq!(row.values, original[expected_index].values);
                assert!(
                    std::ptr::eq(*row, &table.rows()[expected_index]),
                    "Expected a reference to the original row"
                );
            }
            _ => panic!("find_rows should return immutable row references"),
        }
    }
    assert_eq!(table.rows(), original);
    assert_eq!(table.column_map, columns);
    assert_eq!(table.dimensions(), (2, 5));
}

fn assert_mutable_matches<T: Fn(&Row) -> bool>(filter: FilterType<T>, expected: &[usize]) {
    let mut table = sample();
    let original = table.rows().to_vec();
    let columns = table.column_map.clone();
    {
        let matches = table.find_rows_mut(filter).unwrap();
        assert_eq!(matches.len(), expected.len());
        for (result, &expected_index) in matches.into_iter().zip(expected) {
            match result {
                FindRowResultTypes::FindRowResultMutable(row, index) => {
                    assert_eq!(index, expected_index);
                    assert_eq!(row.values, original[expected_index].values);
                    row.values[0] = Value::from(99);
                }
                _ => panic!("find_rows_mut should return mutable row references"),
            }
        }
    }
    for (index, row) in table.rows().iter().enumerate() {
        if expected.contains(&index) {
            assert_eq!(
                row.values,
                vec![Value::from(99), original[index].values[1].clone()]
            );
        } else {
            assert_eq!(row.values, original[index].values);
        }
    }
    assert_eq!(table.column_map, columns);
    assert_eq!(table.dimensions(), (2, 5));
}

#[test]
fn find_empty_rows_returns_original_indices_and_references() {
    assert_immutable_matches(FilterType::<fn(&Row) -> bool>::Empty, &[0, 4]);
}

#[test]
fn find_nonempty_rows_includes_partially_populated_rows() {
    assert_immutable_matches(FilterType::<fn(&Row) -> bool>::NonEmpty, &[1, 2, 3]);
}

#[test]
fn find_conditional_rows_preserves_order_and_original_indices() {
    assert_immutable_matches(
        FilterType::Conditional(|row: &Row| !row.values[0].is_empty()),
        &[1, 3],
    );
}

#[test]
fn find_no_matches_returns_no_rows() {
    assert_immutable_matches(FilterType::Conditional(|_: &Row| false), &[]);
}

#[test]
fn find_all_matches_returns_every_original_row() {
    assert_immutable_matches(FilterType::Conditional(|_: &Row| true), &[0, 1, 2, 3, 4]);
}

#[test]
fn find_rows_accepts_a_captured_predicate() {
    let threshold = 3;
    assert_immutable_matches(
        FilterType::Conditional(|row: &Row| {
            row.values[0]
                .as_integer()
                .is_some_and(|value| value > threshold)
        }),
        &[3],
    );
}

#[test]
fn find_empty_rows_mut_edits_only_matching_original_rows() {
    assert_mutable_matches(FilterType::<fn(&Row) -> bool>::Empty, &[0, 4]);
}

#[test]
fn find_nonempty_rows_mut_includes_partially_populated_rows() {
    assert_mutable_matches(FilterType::<fn(&Row) -> bool>::NonEmpty, &[1, 2, 3]);
}

#[test]
fn find_conditional_rows_mut_edits_matches_at_original_indices() {
    assert_mutable_matches(
        FilterType::Conditional(|row: &Row| !row.values[0].is_empty()),
        &[1, 3],
    );
}

#[test]
fn find_no_matches_mut_leaves_every_row_unchanged() {
    assert_mutable_matches(FilterType::Conditional(|_: &Row| false), &[]);
}

#[test]
fn find_all_matches_mut_edits_every_original_row() {
    assert_mutable_matches(FilterType::Conditional(|_: &Row| true), &[0, 1, 2, 3, 4]);
}

#[test]
fn find_rows_mut_accepts_a_captured_predicate() {
    let threshold = 3;
    assert_mutable_matches(
        FilterType::Conditional(|row: &Row| {
            row.values[0]
                .as_integer()
                .is_some_and(|value| value > threshold)
        }),
        &[3],
    );
}

#[test]
fn finding_rows_in_empty_table_returns_no_matches_for_every_filter() {
    let mut table = empty_table();
    assert!(
        table
            .find_rows(FilterType::<fn(&Row) -> bool>::Empty)
            .unwrap()
            .is_empty()
    );
    assert!(
        table
            .find_rows(FilterType::<fn(&Row) -> bool>::NonEmpty)
            .unwrap()
            .is_empty()
    );
    assert!(
        table
            .find_rows(FilterType::Conditional(|_: &Row| true))
            .unwrap()
            .is_empty()
    );
    assert!(
        table
            .find_rows_mut(FilterType::<fn(&Row) -> bool>::Empty)
            .unwrap()
            .is_empty()
    );
    assert!(
        table
            .find_rows_mut(FilterType::<fn(&Row) -> bool>::NonEmpty)
            .unwrap()
            .is_empty()
    );
    assert!(
        table
            .find_rows_mut(FilterType::Conditional(|_: &Row| true))
            .unwrap()
            .is_empty()
    );
    assert_eq!(table.dimensions(), (2, 0));
}

#[test]
fn mutable_results_allow_multiple_row_references_at_once() {
    let mut table = sample();
    let mut matches = table
        .find_rows_mut(FilterType::<fn(&Row) -> bool>::NonEmpty)
        .unwrap();
    assert_eq!(matches.len(), 3);
    let third = matches.pop().unwrap();
    let second = matches.pop().unwrap();
    let first = matches.pop().unwrap();
    match (first, second, third) {
        (
            FindRowResultTypes::FindRowResultMutable(a, 1),
            FindRowResultTypes::FindRowResultMutable(b, 2),
            FindRowResultTypes::FindRowResultMutable(c, 3),
        ) => {
            a.values[0] = Value::from(10);
            b.values[0] = Value::from(20);
            c.values[0] = Value::from(30);
        }
        _ => panic!("Expected mutable references at original indices 1, 2, 3"),
    }
    assert_eq!(table[1].values, vec![Value::from(10), Value::from(2.5)]);
    assert_eq!(table[2].values, vec![Value::from(20), Value::from(3.5)]);
    assert_eq!(table[3].values, vec![Value::from(30), Value::Empty]);
    assert_eq!(table[0].values, vec![Value::Empty, Value::Empty]);
    assert_eq!(table[4].values, vec![Value::Empty, Value::Empty]);
}
