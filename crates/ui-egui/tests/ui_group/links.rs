//! The simplified shell omits community promotion and the About dialog.

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use pdfcraft_engine::links;
use pdfcraft_ui_egui::PdfCraftApp;

fn harness(setup: impl FnOnce(&mut PdfCraftApp) + 'static) -> Harness<'static, PdfCraftApp> {
    let mut h = Harness::builder().with_size(egui::vec2(1400.0, 900.0)).build_eframe(move |_cc| {
        let mut app = PdfCraftApp::new();
        app.set_option("language", "en").unwrap();
        setup(&mut app);
        app
    });
    h.run_steps(4);
    h
}

#[test]
fn home_and_top_bar_omit_community_promotion() {
    let h = harness(|_| {});
    for label in ["Discord", "Join the ArtCraft community", "Join our Discord", "ArtCraft", "About Readf", "Contributors", "Models"] {
        assert!(h.query_by_label(label).is_none(), "unexpected promotional UI: {label}");
    }
    h.get_by_label("Recommended tools");
    h.get_by_label("Keyboard shortcuts");
}

#[test]
fn removed_commands_and_dialog_cannot_be_opened() {
    let mut h = harness(|_| {});
    for id in ["help.about", "help.discord"] {
        assert!(pdfcraft_engine::commands::command(id).is_none());
        assert!(!h.state_mut().execute(id));
    }
    h.state_mut().execute("help.shortcuts");
    let previous = h.state().dialog;
    assert!(h.state_mut().set_option("dialog", "about").is_err());
    assert_eq!(h.state().dialog, previous, "invalid dialog keeps the existing one");
    assert!(h.state().last_opened_url.is_none());
}

#[test]
fn project_links_are_unavailable_in_the_local_interface() {
    for l in links::LINKS {
        let mut h = harness(|_| {});
        assert!(!h.state_mut().execute(l.command), "{}", l.command);
        assert!(h.state().last_opened_url.is_none());
        assert_eq!(pdfcraft_engine::commands::command(l.command).unwrap().menu, Some("Help"));
    }
}
