use crate::krill::{CountType, KrillErrors, Table, Value, ValueType};
use std::{error::Error, io, path::Path};

const INPUT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/test_data/test input/");

fn sample() -> Table<'static> {
    Table::from_csv(Path::new(concat!(env!("CARGO_MANIFEST_DIR"),
        "/test_data/test input/column_operations_test.csv"))).unwrap()
}

#[test]
fn invalid_csv_inputs_return_specific_errors() {
    for (name, expected) in [
        ("empty.csv", "empty"),
        ("mixed_types.csv", "type"),
        ("extra_fields.csv", "bounds"),
        ("blank_row.csv", "parse"),
    ] {
        let path = Path::new(INPUT).join(name);
        let error = Table::from_csv(&path).unwrap_err();
        let error = error.downcast_ref::<KrillErrors>().unwrap();
        assert!(match (expected, error) {
            ("empty", KrillErrors::EmptyCSV) => true,
            ("type", KrillErrors::ColumnTypeMismatched(ValueType::Integer, ValueType::Float)) => true,
            ("bounds", KrillErrors::ColumnOutOfBounds(2, 2)) => true,
            ("parse", KrillErrors::ParsingError) => true,
            _ => false,
        }, "{name}: unexpected error {error:?}");
    }
}

#[test]
fn missing_file_returns_not_found() {
    let path = Path::new(INPUT).join("intentionally_missing.csv");
    let error = Table::from_csv(&path).unwrap_err();
    assert_eq!(error.downcast_ref::<io::Error>().unwrap().kind(), io::ErrorKind::NotFound);
}

fn assert_missing_column<T: std::fmt::Debug>(result: Result<T, Box<dyn Error>>) {
    let error = result.unwrap_err();
    assert!(matches!(krill_error(error.as_ref()),
        Some(KrillErrors::ColumnNotFound(name)) if name == "missing"));
}

fn krill_error<'a>(error: &'a (dyn Error + 'static)) -> Option<&'a KrillErrors> {
    // Some existing methods wrap Box<KrillErrors> again through `?`.
    error.downcast_ref::<KrillErrors>().or_else(|| {
        error.downcast_ref::<Box<KrillErrors>>().map(Box::as_ref)
    })
}

#[test]
fn aggregates_return_errors_for_missing_columns() {
    let table = sample();
    assert_missing_column(table.sum("missing"));
    assert_missing_column(table.mean("missing"));
    assert_missing_column(table.count("missing", CountType::NonEmpty));
    assert_missing_column(table.median("missing"));
    assert_missing_column(table.mode("missing"));
}

#[test]
fn fillna_returns_missing_column_error() {
    let mut table = sample();
    assert_missing_column(table.fillna_column("missing", |_, _| Ok(Value::from(0)), true));
}

#[test]
fn fillna_propagates_replacement_error() {
    let mut table = sample();
    let error = table.fillna_column("integer_values", |_, _| {
        Err(Box::new(io::Error::new(io::ErrorKind::InvalidData, "replacement failed")))
    }, true).unwrap_err();
    assert_eq!(error.downcast_ref::<io::Error>().unwrap().kind(), io::ErrorKind::InvalidData);
}

#[test]
fn export_without_overwrite_returns_already_exists() {
    // Use an existing input as the destination; create_new must leave it untouched.
    let path = Path::new(INPUT).join("column_operations_test.csv");
    let original = std::fs::read(&path).unwrap();
    let error = sample().to_csv(&path, false).unwrap_err();
    assert_eq!(error.downcast_ref::<io::Error>().unwrap().kind(), io::ErrorKind::AlreadyExists);
    assert_eq!(std::fs::read(&path).unwrap(), original);
}

#[test]
fn row_access_returns_bounds_error() {
    let table = sample();
    let error = table.get_row(table.rows().len()).unwrap_err();
    assert!(matches!(krill_error(error.as_ref()), Some(KrillErrors::RowOutOfBounds(12, 12))));
}

#[test]
#[should_panic(expected = "Failed to index Table")]
fn row_index_panics_out_of_bounds() { let _ = &sample()[12]; }

#[test]
#[should_panic(expected = "Integer Equality Comparison")]
fn float_value_equality_with_i64_panics() { let _ = Value::from(5.0) == 5_i64; }

#[test]
#[should_panic(expected = "Float Equality Comparison")]
fn integer_value_equality_with_f64_panics() { let _ = Value::from(5_i64) == 5.0_f64; }

#[test]
#[should_panic(expected = "Integer Order Comparison")]
fn float_value_ordering_with_i64_panics() { let _ = Value::from(5.0) < 6_i64; }

#[test]
#[should_panic(expected = "Float Order Comparison")]
fn integer_value_ordering_with_f64_panics() { let _ = Value::from(5_i64) < 6.0_f64; }

#[test]
#[should_panic(expected = "not an numeric type")]
fn adding_text_to_number_panics() { let _ = Value::from("text") + Value::from(1); }

#[test]
#[should_panic(expected = "divide by zero")]
fn dividing_by_zero_panics() { let _ = Value::from(5) / 0.0; }

#[test]
#[should_panic(expected = "not an numeric type")]
fn sum_of_text_panics() { let _ = sample().sum("category"); }

#[test]
#[should_panic(expected = "not an numeric type")]
fn mean_of_text_panics() { let _ = sample().mean("category"); }

#[test]
#[should_panic(expected = "divide by zero")]
fn mean_of_all_empty_column_panics() { let _ = sample().mean("all_empty"); }

#[test]
#[should_panic(expected = "Integer Order Comparison")]
fn count_with_wrong_numeric_predicate_panics() {
    let _ = sample().count("float_values", CountType::Conditional(Box::new(|value| *value > 10_i64)));
}

#[test]
fn duplicate_column_additions_are_noops() {
    // These operations currently succeed rather than returning an error.
    let mut table = sample();
    let rows = table.rows().to_vec();
    let dimensions = table.dimensions();
    table.add_column_empty("integer_values", true).unwrap();
    table.add_column_default(Value::from("replacement"), "integer_values", true).unwrap();
    assert_eq!(table.rows(), rows);
    assert_eq!(table.dimensions(), dimensions);
}

#[test]
#[ignore = "Future behavior requested: mixed numeric addition currently promotes to float"]
#[should_panic]
fn integer_value_plus_f64_should_panic() { let _ = Value::from(5_i64) + 0.5_f64; }
