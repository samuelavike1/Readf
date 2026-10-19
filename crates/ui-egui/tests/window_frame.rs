//! Integrated Windows title bar: geometry, native commands and unsaved-work protection.
#![cfg(target_os = "windows")]

use egui::{ViewportCommand, ViewportId, pos2, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use pdfcraft_ui_egui::{PdfCraftApp, window_controls};

const PDF: &[u8] = b"%PDF-1.7
1 0 obj << /Type /Catalog /Pages 2 0 R >> endobj
2 0 obj << /Type /Pages /Kids [3 0 R] /Count 1 >> endobj
3 0 obj << /Type /Page /Parent 2 0 R /MediaBox [0 0 300 400] >> endobj
trailer << /Root 1 0 R >>
%%EOF";

fn harness() -> Harness<'static, PdfCraftApp> {
    let mut h = Harness::builder().with_size(vec2(820.0, 600.0)).with_step_dt(1.0 / 60.0).build_eframe(|_| {
        let mut app = PdfCraftApp::new();
        app.set_option("language", "en").unwrap();
        app.window_controls = Some(window_controls::Layout::default());
        app
    });
    h.run_steps(5);
    h
}

fn emitted(h: &Harness<'static, PdfCraftApp>, check: impl Fn(&ViewportCommand) -> bool) -> bool {
    h.output().viewport_output.values().flat_map(|v| &v.commands).any(check)
}

fn click_emits(h: &mut Harness<'static, PdfCraftApp>, label: &str, check: impl Fn(&ViewportCommand) -> bool) {
    h.query_all_by_label(label).find(|n| n.rect().top() < 38.0).unwrap().click();
    let mut seen = false;
    for _ in 0..4 {
        h.step();
        seen |= emitted(h, &check);
    }
    assert!(seen, "{label} must send its native window command");
}

#[test]
fn window_buttons_stay_at_the_right_edge_with_overflowing_tabs() {
    let mut h = harness();
    for n in 0..8 {
        h.state_mut().open_bytes(&format!("Document {n}.pdf"), None, PDF.to_vec()).unwrap();
    }
    h.run_steps(5);
    for width in [820.0, 1100.0] {
        h.set_size(vec2(width, 600.0));
        h.run_steps(5);
        let minimize = h.get_by_label("Minimize").rect();
        let maximize = h.get_by_label("Maximize").rect();
        let close = h.query_all_by_label("Close").find(|n| n.rect().top() < 38.0).unwrap().rect();
        assert_eq!(close.size(), vec2(46.0, 38.0));
        assert!((close.right() - width).abs() < 1.0, "close must reach the window edge: {close:?}");
        assert!(minimize.right() <= maximize.left() && maximize.right() <= close.left());
        assert!(h.get_by_label_contains("Display theme:").rect().right() < minimize.left());
        assert!(h.get_by_label("Open").rect().right() < minimize.left());
        assert!((close.center().y - h.get_by_label("Menu").rect().center().y).abs() < 1.0);
    }
}

#[test]
fn window_buttons_send_native_commands_and_protect_unsaved_work() {
    let mut h = harness();
    click_emits(&mut h, "Minimize", |c| matches!(c, ViewportCommand::Minimized(true)));
    click_emits(&mut h, "Maximize", |c| matches!(c, ViewportCommand::Maximized(true)));
    h.input_mut().viewports.get_mut(&ViewportId::ROOT).unwrap().maximized = Some(true);
    h.run_steps(3);
    click_emits(&mut h, "Restore", |c| matches!(c, ViewportCommand::Maximized(false)));
    h.state_mut().open_bytes("Draft.pdf", None, PDF.to_vec()).unwrap();
    h.state_mut().apply_edit(pdfcraft_engine::Edit::AddBookmark { parent: vec![], index: 0, title: "Keep me".into(), page: 0 });
    h.run_steps(3);
    click_emits(&mut h, "Close", |c| matches!(c, ViewportCommand::Close));
    // The window backend reports Close back to the app, where the common quit guard runs.
    h.input_mut().viewports.get_mut(&ViewportId::ROOT).unwrap().events.push(egui::ViewportEvent::Close);
    h.step();
    assert!(emitted(&h, |c| matches!(c, ViewportCommand::CancelClose)));
    h.get_by_label("Cancel").click();
    h.run_steps(3);
    let id = h.state().views[0].id;
    assert!(h.state().session.get(id).unwrap().dirty, "cancel must retain unsaved work");
}

#[test]
fn empty_title_bar_drags_and_window_edges_resize() {
    let mut h = harness();
    // Empty space after Open and before the right-side actions.
    h.hover_at(pos2(400.0, 19.0));
    h.run_steps(2);
    h.drag_at(pos2(400.0, 19.0));
    h.step();
    h.hover_at(pos2(440.0, 19.0));
    let mut dragged = false;
    for _ in 0..4 {
        h.step();
        dragged |= emitted(&h, |c| matches!(c, ViewportCommand::StartDrag));
    }
    assert!(dragged, "empty title-bar space must move the native window");
    h.drop_at(pos2(440.0, 19.0));
    h.run_steps(2);
    h.hover_at(pos2(819.0, 300.0));
    h.run_steps(2);
    h.drag_at(pos2(819.0, 300.0));
    let mut resized = false;
    for _ in 0..4 {
        h.step();
        resized |= emitted(&h, |c| matches!(c, ViewportCommand::BeginResize(egui::viewport::ResizeDirection::East)));
    }
    assert!(resized, "the right edge must initiate native resize");
}

#[test]
fn double_clicking_empty_title_bar_toggles_maximize() {
    for maximized in [false, true] {
        let mut h = harness();
        h.input_mut().viewports.get_mut(&ViewportId::ROOT).unwrap().maximized = Some(maximized);
        h.run_steps(3);
        let pos = pos2(400.0, 19.0);
        let mut toggled = false;
        h.event(egui::Event::PointerMoved(pos));
        for _ in 0..2 {
            for pressed in [true, false] {
                h.event(egui::Event::PointerButton { pos, button: egui::PointerButton::Primary, pressed, modifiers: egui::Modifiers::NONE });
                h.step();
                toggled |= emitted(&h, |c| matches!(c, ViewportCommand::Maximized(value) if *value != maximized));
            }
        }
        assert!(toggled, "double-click must toggle maximize from {maximized}");
    }
}
