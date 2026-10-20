//! Layout checks for the simplified PDF Optimizer dialog.

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use pdfcraft_ui_egui::PdfCraftApp;

fn harness(size: egui::Vec2) -> Harness<'static, PdfCraftApp> {
    let mut h = Harness::builder().with_size(size).build_eframe(|_cc| {
        let mut app = PdfCraftApp::new();
        app.set_option("language", "en").unwrap();
        let bytes = app.session.create_from_text("document", "A small document").unwrap().to_vec();
        app.open_bytes("document.pdf", None, bytes).unwrap();
        app.set_option("dialog", "optimize").unwrap();
        app
    });
    h.run_steps(5);
    h
}

fn advanced_starts_collapsed_and_can_be_opened(size: egui::Vec2) {
    let mut h = harness(size);
    h.get_by_label("PDF Optimizer");
    h.get_by_label("Color Images");
    assert!(h.query_by_label("Discard document tags").is_none(), "advanced choices should be hidden initially");
    h.get_by_label("Advanced options").scroll_to_me();
    h.run_steps(8);
    h.get_by_label("Advanced options").click();
    h.run_steps(2);
    h.get_by_label("Discard Objects");
    h.get_by_label("Discard document tags");
    let action = h.get_by_label("OK").rect();
    assert!(action.bottom() <= size.y, "the inline action escaped the viewport: {action:?} in {size:?}");
}

#[test]
fn optimizer_advanced_options_are_collapsed_but_reachable_at_wide_size() {
    advanced_starts_collapsed_and_can_be_opened(egui::vec2(1100.0, 800.0));
}

#[test]
fn optimizer_keeps_inline_actions_reachable_at_narrow_size() {
    advanced_starts_collapsed_and_can_be_opened(egui::vec2(420.0, 650.0));
}
