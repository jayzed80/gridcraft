//! Format Cells dialog stays within a narrow window even when a long number format is selected.

use egui::vec2;
use egui_kittest::kittest::Queryable as _;
use gridcraft_engine::Session;
use gridcraft_ui_egui::SheetApp;
use serde_json::json;

type Harness = egui_kittest::Harness<'static, SheetApp>;

fn narrow_accounting_harness(viewport_width: f32) -> Harness {
    let mut session = Session::new();
    session.new_workbook();
    session.execute("cell.set", json!({"cell": "A1", "input": "1850"})).unwrap();
    session.execute("home.numberFormat", json!({"range": "A1", "format": "Accounting"})).unwrap();
    let mut h = crate::common::builder().with_size(vec2(viewport_width, 800.0)).build_ui_state(
        |ui, app: &mut SheetApp| {
            app.logic(&ui.ctx().clone());
            app.ui(ui);
        },
        crate::common::app(session),
    );
    h.run_steps(4);
    h
}

#[test]
fn format_cells_stays_inside_narrow_window_for_accounting_format() {
    // 412 CSS px at 85% interface size is ~485 logical px; also check 412 logical px directly.
    for viewport_width in [485.0, 412.0] {
        let mut h = narrow_accounting_harness(viewport_width);
        h.state_mut().open_dialog("formatCells", json!({"tab": "Number"}));
        h.run_steps(4);

        let screen = h.ctx.content_rect();
        let dialog = h.ctx.memory(|m| m.area_rect(egui::Id::unique("dialog").with("formatCells"))).unwrap();
        let category_general = h.get_by_label("General").rect();
        let ok = h.get_by_label_contains("OK").rect();
        let cancel = h.get_by_label(" Cancel ").rect();

        assert!(
            dialog.left() >= screen.left() && dialog.right() <= screen.right(),
            "viewport {viewport_width}: Format Cells dialog {dialog:?} overflows {screen:?}"
        );
        assert!(
            category_general.left() >= screen.left(),
            "viewport {viewport_width}: Category list {category_general:?} clipped on left of {screen:?}"
        );
        assert!(
            ok.right() <= screen.right() && cancel.right() <= screen.right(),
            "viewport {viewport_width}: OK {ok:?} / Cancel {cancel:?} clipped on right of {screen:?}"
        );
    }
}
