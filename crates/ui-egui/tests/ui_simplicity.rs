//! Regression coverage for the simplified reading UI and its tool/sidebar transitions.

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use pdfcraft_ui_egui::{PdfCraftApp, RightPanel};

const PDF_WITH_COMMENT: &[u8] = b"%PDF-1.7
1 0 obj << /Type /Catalog /Pages 2 0 R >> endobj
2 0 obj << /Type /Pages /Kids [3 0 R] /Count 1 >> endobj
3 0 obj << /Type /Page /Parent 2 0 R /MediaBox [0 0 300 200] /Annots [4 0 R] >> endobj
4 0 obj << /Type /Annot /Subtype /Text /Rect [10 160 30 190] /T (Tester) /Contents (Check this) >> endobj
trailer << /Root 1 0 R >>
%%EOF";

fn harness(width: f32, setup: impl FnOnce(&mut PdfCraftApp) + 'static) -> Harness<'static, PdfCraftApp> {
    let mut h = Harness::builder().with_size(egui::vec2(width, 800.0)).build_eframe(move |_cc| {
        let mut app = PdfCraftApp::new();
        app.set_option("language", "en").unwrap();
        app.open_bytes("document.pdf", None, PDF_WITH_COMMENT.to_vec()).unwrap();
        setup(&mut app);
        app
    });
    h.run_steps(5);
    h
}

#[test]
fn tools_search_filters_to_the_matching_catalogue_group_and_opens_edit_pdf() {
    let mut h = harness(1200.0, |_| {});
    h.get_by_label("Tools").click();
    h.run_steps(2);

    h.get_by_label("Search tools").click();
    h.run_steps(2);
    let search = h.get_by_label("Search tools");
    search.type_text("watermark");
    h.run_steps(2);

    h.get_by_label("Edit a PDF");
    assert!(h.query_by_label("Add comments").is_none(), "unrelated tool group leaked into filtered results");
    h.get_by_label("Edit a PDF").click();
    h.run_steps(2);
    assert_eq!(h.state().left, pdfcraft_ui_egui::LeftPanel::Tool("edit"));
    assert!(h.state().left_open);
}

#[test]
fn choosing_a_tool_and_then_a_right_panel_close_the_other_workspace() {
    let mut h = harness(1200.0, |app| {
        app.choose_right_panel(Some(RightPanel::Comments));
        app.left_open = true;
    });
    h.state_mut().set_option("tool", "edit").unwrap();
    h.run_steps(2);
    assert_eq!(h.state().right, None, "choosing a tool closes the reading panel");

    h.state_mut().left_open = true;
    h.state_mut().choose_right_panel(Some(RightPanel::Comments));
    h.run_steps(2);
    assert!(!h.state().left_open, "choosing Comments closes the tool panel");
}

#[test]
fn comments_header_keeps_title_and_count_clear_of_search_and_filter_at_narrow_width() {
    let mut h = harness(1100.0, |app| {
        app.left_open = false;
        app.choose_right_panel(Some(RightPanel::Comments));
    });
    h.run_steps(2);

    let title = h
        .get_all_by_label("Comments")
        .map(|node| node.rect())
        .filter(|rect| rect.left() < 400.0 && rect.top() > 100.0)
        .min_by(|a, b| a.top().total_cmp(&b.top()))
        .expect("comments header title");
    let count = h
        .get_all_by_label("1")
        .map(|node| node.rect())
        .filter(|rect| (rect.center().y - title.center().y).abs() < 16.0 && rect.left() < 400.0)
        .min_by(|a, b| a.left().total_cmp(&b.left()))
        .expect("comments count");
    let search = h.get_by_label("Search comments").rect();
    let filter = h.get_by_label("Filter comments").rect();

    assert!(!title.intersects(search), "title overlaps search: {title:?} {search:?}");
    assert!(!count.intersects(search), "count overlaps search: {count:?} {search:?}");
    assert!(search.right() <= filter.left() || filter.right() <= search.left(), "search and filter overlap: {search:?} {filter:?}");
}
