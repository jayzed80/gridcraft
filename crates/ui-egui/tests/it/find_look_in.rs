//! Find searches the formula text by default, as Excel does, and its "Look in" box switches the
//! search to the values cells show, so the text a CONCATENATE formula makes is found there (#4479).

use egui::accesskit::Role;
use egui::{Event, Key, Modifiers};
use egui_kittest::kittest::Queryable;
use gridcraft_ui_egui::SheetApp;
use serde_json::json;

type Harness = egui_kittest::Harness<'static, SheetApp>;

/// Ctrl on Windows and Linux sets both `ctrl` and `command`.
const CTRL: Modifiers = Modifiers { alt: false, ctrl: true, shift: false, mac_cmd: false, command: true };

/// A1 and B1 joined by C1's formula, whose text "HelloWorld" appears nowhere in its source.
fn harness() -> Harness {
    let mut h = crate::support::blank_harness(6);
    h.state_mut().session.execute("range.setValues", json!({"range": "A1:B1", "values": [["Hello", "World"]]})).unwrap();
    h.state_mut().session.execute("cell.set", json!({"cell": "C1", "input": "=CONCATENATE(A1,B1)"})).unwrap();
    h.run_steps(2);
    h
}

/// Opens Find and Replace with Ctrl+F and types `what` into "Find what".
fn open_find(h: &mut Harness, what: &str) {
    h.input_mut().events.push(Event::Key { key: Key::F, physical_key: Some(Key::F), pressed: true, repeat: false, modifiers: CTRL });
    h.input_mut().events.push(Event::Key { key: Key::F, physical_key: Some(Key::F), pressed: false, repeat: false, modifiers: CTRL });
    h.run_steps(6);
    h.input_mut().events.push(Event::Text(what.into()));
    h.run_steps(4);
}

fn find_next(h: &mut Harness) {
    h.get_by_label("Find Next").click();
    h.run_steps(4);
}

/// Picks a "Look in" choice from the dialog's list.
fn look_in(h: &mut Harness, choice: &str) {
    h.get_by_role_and_label(Role::ComboBox, "Look in:").click();
    h.run_steps(3);
    h.get_all_by_label(choice).last().expect("the choice is listed").click();
    h.run_steps(3);
}

#[test]
fn the_value_a_formula_makes_is_found_with_look_in_values() {
    let mut h = harness();
    open_find(&mut h, "HelloWorld");
    // The default is Formulas, as in Excel: the concatenated text is not in the formula's source.
    find_next(&mut h);
    assert!(h.state().message.is_some(), "Formulas does not find the value");
    h.state_mut().message = None;
    look_in(&mut h, "Values");
    find_next(&mut h);
    assert!(h.state().message.is_none(), "{:?}", h.state().message);
    assert_eq!(h.state().session.active().map(|d| d.selection.active.a1()), Some("C1".to_string()));
}

#[test]
fn look_in_formulas_still_finds_the_formula_text() {
    let mut h = harness();
    open_find(&mut h, "CONCATENATE");
    look_in(&mut h, "Formulas");
    find_next(&mut h);
    assert!(h.state().message.is_none(), "{:?}", h.state().message);
    assert_eq!(h.state().session.active().map(|d| d.selection.active.a1()), Some("C1".to_string()));
    assert_eq!(h.state().dialog.as_ref().map(|d| d.values.get("lookIn").cloned()), Some(Some(json!("formulas"))));
}
