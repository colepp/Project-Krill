# Test inputs

Inputs live here; generated CSV exports remain in `../test_outputs/`.

| File | Contents and purpose |
| --- | --- |
| column_operations_test.csv | Numeric, text, boolean, and empty cells matching the existing aggregate assertions. |
| baseball_stats.csv | Sample baseball records with missing averages for fill operations. |
| empty.csv | Zero-byte file; tests the EmptyCSV error. |
| mixed_types.csv | Integer followed by float in one column; tests strict column typing. |
| extra_fields.csv | More values than headers; tests column bounds validation. |
| blank_row.csv | Empty line between header and data; tests the parsing error. |
| header_only.csv | Named columns without records; tests undefined mean behavior and future empty-table handling. |
| short_rows.csv | Missing trailing field; reserved for future row-width validation. |
| duplicate_headers.csv | Repeated column name; reserved for future header validation. |
| quoted_fields.csv | Commas, escaped quotes, and multiline fields; reserved for future CSV parsing support. |
| median_cases.csv | Five odd-column values and four even-column values plus blanks; future missing-value median tests should expect 5.0 in both columns. |
| numeric_limits.csv | i64 boundaries and large finite floats; reserved for future overflow tests. |

Future fixtures do not imply that the current implementation supports or rejects them correctly.
Failure tests are in `src/krill_testing/failure_cases.rs`; the original tests remain
in `src/krill_testing.rs`. Library implementation files are unchanged.

Mixed numeric addition currently promotes to float. The requested panic test is
ignored pending that behavior change. Primitive Rust `i64 + f64` is a compile-time
type error, not a runtime panic.
