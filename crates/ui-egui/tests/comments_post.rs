//! Small regression tests for the simple Comments panel composer.

use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use pdfcraft_ui_egui::PdfCraftApp;

const PDF: &[u8] = b"%PDF-1.7
1 0 obj << /Type /Catalog /Pages 2 0 R >> endobj
2 0 obj << /Type /Pages /Kids [3 0 R] /Count 1 >> endobj
3 0 obj << /Type /Page /Parent 2 0 R /MediaBox [0 0 300 200] >> endobj
trailer << /Root 1 0 R >>
%%EOF";

fn harness(size: egui::Vec2) -> Harness<'static, PdfCraftApp> {
    let mut h = Harness::builder().with_size(size).build_eframe(|_cc| {
        let mut app = PdfCraftApp::new();
        app.set_option("language", "en").unwrap();
        app.set_option("author", "Tester").unwrap();
        app.open_bytes("comments.pdf", None, PDF.to_vec()).unwrap();
        app.set_option("panel", "comments").unwrap();
        app
    });
    h.run_steps(4);
    h
}

fn annotation_count(h: &Harness<'static, PdfCraftApp>) -> usize {
    let state = h.state();
    state.session.get(state.views[0].id).map_or(0, |doc| doc.info.annotations.len())
}

#[test]
fn post_creates_a_comment_from_the_top_composer() {
    let mut h = harness(egui::vec2(900.0, 700.0));
    h.query_all_by(|n| n.placeholder() == Some("Add a comment")).next().expect("composer field").click();
    h.run_steps(2);
    h.query_all_by(|n| n.placeholder() == Some("Add a comment")).next().expect("composer field").type_text("Review this page");
    h.run_steps(2);
    h.get_by_label("Post comment").click();
    h.run_steps(3);
    let state = h.state();
    let doc = state.session.get(state.views[0].id).expect("document");
    assert_eq!(doc.info.annotations.len(), 1);
    assert_eq!(doc.info.annotations[0].contents.as_deref(), Some("Review this page"));
}

#[test]
fn blank_post_is_disabled_and_does_not_create_a_comment() {
    let mut h = harness(egui::vec2(900.0, 700.0));
    let post = h.get_by_label("Post comment");
    assert!(post.accesskit_node().is_disabled(), "blank composer must keep Post disabled");
    post.click();
    h.run_steps(2);
    assert_eq!(annotation_count(&h), 0);
}

#[test]
fn composer_controls_stay_inside_a_narrow_sidebar() {
    let mut h = harness(egui::vec2(900.0, 700.0));
    let panel_id = egui::Id::new("read_right_panel");
    let mut panel = egui::containers::panel::PanelState::load(&h.ctx, panel_id).expect("reading sidebar");
    panel.outer_rect.max.x = panel.outer_rect.left() + 200.0;
    h.ctx.data_mut(|data| data.insert_persisted(panel_id, panel));
    h.run_steps(5);
    let bounds = egui::containers::panel::PanelState::load(&h.ctx, panel_id).expect("reading sidebar").outer_rect;
    assert!(bounds.width() <= 201.0, "content expanded the 200px sidebar: {bounds:?}");
    let field = h.query_all_by(|n| n.placeholder() == Some("Add a comment")).next().expect("composer field");
    let post = h.get_by_label("Post comment");
    assert!(field.rect().left() >= bounds.left() && field.rect().right() <= bounds.right(), "field escapes sidebar: {:?}", field.rect());
    assert!(post.rect().left() >= bounds.left() && post.rect().right() <= bounds.right(), "Post escapes sidebar: {:?}", post.rect());
    assert!(field.rect().right() <= post.rect().left(), "composer field overlaps Post");
}

#[test]
fn reading_mode_shows_styles_only_while_an_annotation_tool_is_active() {
    let mut h = harness(egui::vec2(1100.0, 800.0));
    assert_eq!(h.state().mode, pdfcraft_ui_egui::Mode::Read);
    assert!(h.query_by_label("Colour").is_none());
    h.state_mut().set_option("quick", "highlight").unwrap();
    h.run_steps(6);
    h.get_by_label("Colour");
    h.state_mut().set_option("quick", "select").unwrap();
    h.run_steps(6);
    assert!(h.query_by_label("Colour").is_none());
}
