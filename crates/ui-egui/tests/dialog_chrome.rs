//! Dialog cards keep their padding, and text fields are tall enough to use.

use egui::accesskit::Role;
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
    for (i, o) in objs.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n{o}\nendobj\n", i + 1).as_bytes());
    }
    let xref = out.len();
    out.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", objs.len() + 1).as_bytes());
    for o in offsets {
        out.extend_from_slice(format!("{o:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n", objs.len() + 1).as_bytes());
    out
}

fn card(h: &Harness<'_, PdfCraftApp>, id: &str) -> egui::Rect {
    h.ctx.data(|d| d.get_temp(egui::Id::new(id).with("card"))).unwrap_or_else(|| panic!("missing dialog card {id}"))
}

fn protected() -> Vec<u8> {
    let mut doc = pdfcraft_cos::Document::open(std::sync::Arc::new(fixture())).unwrap();
    doc.set_encryption(&pdfcraft_cos::NewEncryption {
        algorithm: pdfcraft_cos::Algorithm::Aes256,
        user_password: "pw",
        owner_password: "owner",
        permissions: -1,
        encrypt_metadata: true,
        seed: [4; 32],
    })
    .unwrap();
    pdfcraft_cos::write_full(&doc, &Default::default()).unwrap()
}

#[test]
fn document_properties_fields_are_tall_and_inset() {
    let mut h = Harness::builder().with_size(egui::vec2(1400.0, 900.0)).build_eframe(|_cc| {
        let mut app = PdfCraftApp::new();
        app.set_option("language", "en").unwrap();
        app.open_bytes("doc.pdf", None, fixture()).unwrap();
        app.set_option("dialog", "properties").unwrap();
        app
    });
    h.run_steps(4);
    let card = card(&h, "dialog");
    let heading = h.get_by_label("Document Properties").rect();
    assert!(heading.left() - card.left() >= 16.0, "heading inset {heading:?} in {card:?}");
    assert!(heading.top() - card.top() >= 16.0, "heading inset {heading:?} in {card:?}");
    let field = h.get_by_role_and_label(Role::TextInput, "Title").rect();
    assert!(field.height() >= 30.0, "title field {field:?}");
    assert!(field.left() >= card.left() + 16.0, "title field {field:?} in {card:?}");
    assert!(field.right() <= card.right() - 16.0, "title field {field:?} in {card:?}");
}

#[test]
fn protection_dialog_shrinks_after_preferences() {
    let mut h = Harness::builder().with_size(egui::vec2(1100.0, 800.0)).build_eframe(|_cc| {
        let mut app = PdfCraftApp::new();
        app.set_option("language", "en").unwrap();
        app.open_bytes("doc.pdf", None, fixture()).unwrap();
        app.set_option("dialog", "preferences").unwrap();
        app
    });
    h.run_steps(5);
    h.state_mut().set_option("dialog", "protect").unwrap();
    h.run_steps(5);
    let card = card(&h, "dialog");
    assert!(card.height() < 500.0, "protection retained the previous dialog's height: {card:?}");
    let cancel = h.get_by_label("Cancel").rect();
    assert!(card.bottom() - cancel.bottom() < 40.0, "actions left blank space in {card:?}: {cancel:?}");
}

#[test]
fn print_and_ocr_cards_fit_short_and_narrow_windows() {
    for size in [egui::vec2(1100.0, 800.0), egui::vec2(420.0, 650.0)] {
        for dialog in ["print", "recognize-text"] {
            let mut h = Harness::builder().with_size(size).build_eframe(|_cc| {
                let mut app = PdfCraftApp::new();
                app.set_option("language", "en").unwrap();
                app.open_bytes("doc.pdf", None, fixture()).unwrap();
                if dialog == "print" {
                    assert!(app.execute("print.dialog"));
                } else {
                    app.set_option("dialog", dialog).unwrap();
                }
                app
            });
            h.run_steps(6);
            let card = card(&h, "dialog");
            assert!(card.left() >= 0.0 && card.right() <= size.x, "{dialog} exceeds width {size:?}: {card:?}");
            assert!(card.top() >= 0.0 && card.bottom() <= size.y, "{dialog} exceeds height {size:?}: {card:?}");
            if dialog == "print" {
                if size.x > 1000.0 {
                    let cancel = h.get_by_label("Cancel").rect();
                    assert!(cancel.bottom() <= card.bottom() - 16.0, "print actions need scrolling in a normal window: {cancel:?}");
                }
                use pdfcraft_ui_egui::PrintHandling;
                for handling in [PrintHandling::Poster, PrintHandling::Multiple, PrintHandling::Booklet] {
                    h.state_mut().print_draft.handling = handling;
                    h.run_steps(6);
                    let bounds = h.ctx.data(|d| d.get_temp::<egui::Rect>(egui::Id::new("dialog").with("card"))).unwrap();
                    assert!(bounds.left() >= 0.0 && bounds.right() <= size.x, "{handling:?} exceeds width {size:?}: {bounds:?}");
                    assert!(bounds.top() >= 0.0 && bounds.bottom() <= size.y, "{handling:?} exceeds height {size:?}: {bounds:?}");
                }
            }
        }
    }
}

#[test]
fn password_prompt_field_is_tall_and_inset() {
    let mut h = Harness::builder().with_size(egui::vec2(1400.0, 900.0)).build_eframe(|_cc| {
        let mut app = PdfCraftApp::new();
        app.set_option("language", "en").unwrap();
        app.open_bytes("secret.pdf", None, protected()).unwrap();
        app
    });
    h.run_steps(4);
    let card = card(&h, "password");
    let field = h.get_by_role(Role::PasswordInput).rect();
    assert!(field.height() >= 30.0, "password field {field:?}");
    assert!(field.left() - card.left() >= 16.0, "password field {field:?} in {card:?}");
    assert!(card.right() - field.right() >= 16.0, "password field {field:?} in {card:?}");
    let heading = h.get_by_label("Password required").rect();
    assert!(heading.top() - card.top() >= 16.0, "heading inset {heading:?} in {card:?}");
}
