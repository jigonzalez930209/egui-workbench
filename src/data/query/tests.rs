//! Query language tests.

use super::*;
use crate::data::Column;

fn data() -> Dataset {
    Dataset::from_columns([
        Column::floats("voltage", vec![0.1, 0.4, 0.2, 0.5]),
        Column::ints("cycle", vec![1, 2, 3, 4]),
        Column::text("state", ["idle", "run", "run", "idle"]),
    ])
}

#[test]
fn parses_and_filters_comparisons() {
    let query = Query::parse("voltage > 0.3 && cycle in 2..5").expect("query");
    assert_eq!(query.matching_rows(&data()), [1, 3]);
    let query = Query::parse("state == \"run\" && !(cycle < 3)").expect("query");
    assert_eq!(query.matching_rows(&data()), [2]);
}

#[test]
fn memberships_and_booleans() {
    let query = Query::parse("cycle in [1, 4] || state == 'run'").expect("query");
    assert_eq!(query.matching_rows(&data()), [0, 1, 2, 3]);
    let query = Query::parse("voltage").expect("query");
    assert_eq!(query.matching_rows(&data()), [0, 1, 2, 3]);
    let query = Query::parse("missing == null").expect("query");
    assert_eq!(query.matching_rows(&data()).len(), 4);
}

#[test]
fn errors_have_offsets() {
    let error = Query::parse("voltage >").expect_err("missing value");
    assert!(error.offset() > 0);
    assert!(Query::parse("voltage = 1").is_err());
    assert!(Query::parse("(voltage > 1").is_err());
    assert!(Query::parse("voltage > 1 &&").is_err());
}

#[test]
fn text_and_ranges_compare_by_value() {
    assert!(values_equal(&DataValue::Int(1), &DataValue::Float(1.0)));
    assert!(compare(&DataValue::Text("b".to_owned()), &DataValue::Text("a".to_owned())).is_some());
    let query = Query::parse("state in [\"run\"]").expect("query");
    assert_eq!(query.matching_rows(&data()), [1, 2]);
}
