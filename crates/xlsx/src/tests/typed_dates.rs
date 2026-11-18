//! Typed dates (`t="d"`, ISO 8601) that no calendar has: the value the file supplied must not
//! roll into the next month (#4456).

use gridcraft_core::{CellRef, Value};

use super::minimal;
use crate::{read_xlsx, write_xlsx};

/// 2023-03-01 and 2024-02-29 in the 1900 date system: the values the invalid dates must not
/// turn into, and a real leap day that must keep its value.
const MARCH_1_2023: f64 = 44_986.0;
const LEAP_DAY_2024: f64 = 45_351.0;

fn sheet() -> Vec<u8> {
    minimal(
        r#"<sheetData>
          <row r="1"><c r="A1" t="d"><v>2023-02-29</v></c><c r="B1"><v>42</v></c></row>
          <row r="2"><c r="A2" t="d"><v>2024-04-31</v></c></row>
          <row r="3"><c r="A3" t="d"><v>2024-02-29</v></c></row>
          <row r="4"><c r="A4" t="d"><v>2023-03-01</v></c></row>
          <row r="5"><c r="A5" t="d"><v>2023-02-29T10:30:00</v></c></row>
        </sheetData>"#,
        &[],
        "",
        "",
    )
}

#[test]
fn a_typed_date_that_does_not_exist_keeps_its_text() {
    let (wb, rep) = read_xlsx(&sheet()).unwrap();
    let s = wb.sheet(0).unwrap();
    let at = |a: &str| s.value(CellRef::parse(a).unwrap());
    // 29 February 2023 and 31 April 2024 do not exist. DATE() would roll them into March and
    // May; the cell keeps the text the file supplied instead.
    assert_eq!(at("A1"), Value::text("2023-02-29"));
    assert_eq!(at("A2"), Value::text("2024-04-31"));
    assert_eq!(at("A5"), Value::text("2023-02-29T10:30:00"));
    // Real dates keep their values: the leap day, and the day the first invalid date would
    // have rolled into.
    assert_eq!(at("A3"), Value::Number(LEAP_DAY_2024));
    assert_eq!(at("A4"), Value::Number(MARCH_1_2023));
    // The sentinel is untouched.
    assert_eq!(at("B1"), Value::Number(42.0));
    // The warning names the problem and says how the values were handled.
    assert!(rep.warnings.iter().any(|w| w.contains("2023-02-29") && w.contains("does not exist") && w.contains("text")), "{:?}", rep.warnings);
}

#[test]
fn a_typed_date_that_does_not_exist_survives_a_round_trip() {
    let (wb, _) = read_xlsx(&sheet()).unwrap();
    let (again, _) = read_xlsx(&write_xlsx(&wb).unwrap()).unwrap();
    let s = again.sheet(0).unwrap();
    assert_eq!(s.value(CellRef::parse("A1").unwrap()), Value::text("2023-02-29"));
    assert_eq!(s.value(CellRef::parse("A2").unwrap()), Value::text("2024-04-31"));
    assert_eq!(s.value(CellRef::parse("B1").unwrap()), Value::Number(42.0));
}
