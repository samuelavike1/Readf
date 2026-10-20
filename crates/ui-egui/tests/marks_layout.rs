//! Responsive containment checks for the mark editor dialogs.

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use pdfcraft_ui_egui::PdfCraftApp;

fn fixture() -> Vec<u8> {
    let objs = [
        "<< /Type /Catalog /Pages 2 0 R >>",
        "<< /Type /Pages /Kids [3 0 R] /Count 1 /MediaBox [0 0 200 300] >>",
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 300] >>",
    ];
    let mut out = b"%PDF-1.7\n".to_vec();
    let mut offsets = Vec::new();
    for (i, object) in objs.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n{object}\nendobj\n", i + 1).as_bytes());
    }
    let xref = out.len();
    out.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", objs.len() + 1).as_bytes());
    for offset in offsets {
        out.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n", objs.len() + 1).as_bytes());
    out
}

fn card(h: &Harness<'_, PdfCraftApp>) -> egui::Rect {
    h.ctx.data(|d| d.get_temp(egui::Id::new("dialog").with("card"))).expect("mark dialog card")
}

#[test]
fn mark_dialogs_stay_contained_at_normal_and_short_sizes() {
    for size in [egui::vec2(1100.0, 800.0), egui::vec2(420.0, 650.0)] {
        for dialog in ["header-footer", "watermark", "background"] {
            let mut h = Harness::builder().with_size(size).build_eframe(move |_cc| {
                let mut app = PdfCraftApp::new();
                app.set_option("language", "en").unwrap();
                app.open_bytes("doc.pdf", None, fixture()).unwrap();
                app.set_option("dialog", dialog).unwrap();
                app
            });
            h.run_steps(6);
            if dialog != "background"
                && let Some(advanced) = h.query_by_label("Advanced")
            {
                advanced.scroll_to_me();
                h.run_steps(8);
                h.get_by_label("Advanced").click();
                h.run_steps(8);
                h.get_by_label(if dialog == "watermark" { "Rotation" } else { "Top" });
            }
            let bounds = card(&h);
            assert!(bounds.left() >= 0.0 && bounds.right() <= size.x, "{dialog} exceeds width {size:?}: {bounds:?}");
            assert!(bounds.top() >= 0.0 && bounds.bottom() <= size.y, "{dialog} exceeds height {size:?}: {bounds:?}");
            h.get_by_label("Cancel");
            if dialog != "header-footer" {
                h.state_mut().marks_draft.use_file = true;
                h.state_mut().marks_draft.file = Some(("a-very-long-source-name-".repeat(20), std::sync::Arc::new(fixture())));
                h.run_steps(6);
                let bounds = card(&h);
                assert!(bounds.left() >= 0.0 && bounds.right() <= size.x, "{dialog} file controls exceed width: {bounds:?}");
                assert!(bounds.top() >= 0.0 && bounds.bottom() <= size.y, "{dialog} file controls exceed height: {bounds:?}");
            }
        }
    }
}

#[test]
fn watermark_can_shrink_after_expanding_preview_and_advanced_options() {
    let mut h = Harness::builder().with_size(egui::vec2(1100.0, 800.0)).build_eframe(|_cc| {
        let mut app = PdfCraftApp::new();
        app.set_option("language", "en").unwrap();
        app.open_bytes("doc.pdf", None, fixture()).unwrap();
        app.set_option("dialog", "watermark").unwrap();
        app
    });
    h.run_steps(6);
    h.get_by_label("Preview").click();
    h.run_steps(6);
    h.get_by_label("Advanced").click();
    h.run_steps(6);
    let size = egui::vec2(820.0, 600.0);
    h.set_size(size);
    h.run_steps(10);
    let bounds = card(&h);
    assert!(bounds.top() >= 0.0 && bounds.bottom() <= size.y, "expanded dialog did not shrink: {bounds:?}");
}
