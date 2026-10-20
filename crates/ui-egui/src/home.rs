//! A quiet welcome page: open a PDF and return to recent documents.

use crate::theme::{self, Tokens};
use crate::{PdfCraftApp, icons, panels::human_size, widgets};
use egui::{Align2, CornerRadius, Rect, Sense, Stroke, vec2};

pub fn show(app: &mut PdfCraftApp, ui: &mut egui::Ui) {
    let t = Tokens::get(ui.ctx());
    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        let width = (ui.available_width() - 40.0).clamp(0.0, 680.0);
        let inset = ((ui.available_width() - width) * 0.5).max(0.0);
        ui.horizontal(|ui| {
            ui.add_space(inset);
            ui.vertical(|ui| {
                ui.set_width(width);
                ui.add_space(48.0);
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new(tl!("Welcome to Readf")).font(theme::semibold(32.0)).color(t.text));
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new(tl!("Open a PDF to get started.")).font(theme::regular(17.0)).color(t.text_muted));
                    ui.add_space(30.0);
                    let button_width = width.min(440.0);
                    let (rect, resp) = ui.allocate_exact_size(vec2(button_width, 60.0), Sense::click());
                    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tl!("Open file")));
                    let press = theme::Press::track(ui, &resp);
                    ui.painter().rect_filled(rect, CornerRadius::same(theme::RADIUS_CONTROL), press.primary_fill(&t));
                    let label = ui.painter().layout_no_wrap(tl!("Open file").to_owned(), theme::medium(19.0), t.primary_text);
                    let start = rect.center() - vec2((label.size().x + 40.0) * 0.5, 0.0);
                    icons::paint(ui, Rect::from_center_size(start + vec2(13.0, 0.0), vec2(26.0, 26.0)), "folder-open", 26.0, t.primary_text);
                    ui.painter().galley(start + vec2(40.0, -label.size().y * 0.5), label, t.primary_text);
                    widgets::focus_outline(ui, &resp);
                    if theme::hand(resp).clicked() {
                        app.open_dialog();
                    }
                    ui.add_space(12.0);
                    ui.label(egui::RichText::new(tl!("or drop a PDF here")).color(t.text_muted));
                });
                ui.add_space(44.0);
                let mut clear = false;
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(tl!("Recent files")).font(theme::semibold(17.0)));
                    if !app.recent.is_empty() {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let resp = widgets::ghost_button(ui, "trash-2", tl!("Clear list"));
                            resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tl!("Clear Recent Files")));
                            clear = resp.on_hover_text(tl!("Clear Recent Files")).clicked();
                        });
                    }
                });
                ui.add_space(8.0);
                if app.recent.is_empty() {
                    ui.label(egui::RichText::new(tl!("Files you open in Readf appear here. Drop a PDF anywhere to open it.")).color(t.text_muted));
                }
                let mut open = None;
                let mut remove = None;
                for r in &app.recent {
                    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 54.0), Sense::click());
                    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, &r.name));
                    ui.painter().line_segment([rect.left_bottom(), rect.right_bottom()], Stroke::new(1.0, t.divider));
                    let press = theme::Press::track(ui, &resp);
                    // The row, not its response: the pointer stays "in" the row over its remove button.
                    let hovered = ui.rect_contains_pointer(rect);
                    if hovered && press.hover < 0.04 && press.down < 0.04 {
                        // The remove button sits in the row, so the row response may not be hovered.
                        ui.painter().rect_filled(rect, CornerRadius::same(theme::RADIUS_CONTROL), t.hover);
                    } else {
                        press.wash(ui, rect, theme::RADIUS_CONTROL, false);
                    }
                    {
                        // Remove just this file from the list (#430); the file itself is untouched.
                        let x_rect = Rect::from_center_size(rect.right_center() - vec2(26.0, 0.0), vec2(28.0, 28.0));
                        let x = ui.interact(x_rect, ui.id().with(("remove-recent", &r.path)), Sense::click());
                        let label = crate::i18n::fmt(tl!("Remove {name} from Recent"), &[("name", &r.name)]);
                        x.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, &label));
                        if x.hovered() {
                            ui.painter().rect_filled(x_rect, CornerRadius::same(theme::RADIUS_CONTROL), t.pressed);
                        }
                        if hovered || resp.has_focus() || x.has_focus() {
                            icons::paint(ui, x_rect.shrink(6.0), "x", 16.0, t.icon);
                        }
                        widgets::focus_outline(ui, &x);
                        if x.on_hover_text(tl!("Remove from Recent")).clicked() {
                            remove = Some(r.path.clone());
                        }
                    }
                    let body = rect.translate(press.offset());
                    icons::paint(ui, Rect::from_min_size(body.min + vec2(10.0, 11.0), vec2(24.0, 24.0)), "file-text", 22.0, t.icon);
                    // Room on the right for the remove button.
                    let detail = ui.painter().text(
                        body.right_center() - vec2(48.0, 0.0),
                        Align2::RIGHT_CENTER,
                        format!("{}  ·  {}", crate::i18n::trn(crate::i18n::current(), r.pages as u64, "{n} page", "{n} pages"), human_size(r.size)),
                        theme::regular(12.0),
                        t.text_muted,
                    );
                    // The name and path stop short of the page count; hovering shows the whole path.
                    let width = detail.left() - 16.0 - (body.left() + 46.0);
                    widgets::row_text(ui, body.min + vec2(46.0, 15.0), crate::bidi::visual(&r.name), theme::medium(13.5), t.text, width);
                    widgets::row_text(ui, body.min + vec2(46.0, 32.0), crate::bidi::visual(&r.path), theme::regular(11.0), t.text_faint, width);
                    widgets::focus_outline(ui, &resp);
                    if theme::hand(resp).on_hover_text(&r.path).clicked() {
                        open = Some(r.path.clone());
                    }
                }
                if let Some(p) = open {
                    app.open_recent(&p);
                }
                if let Some(p) = remove {
                    app.recent.retain(|r| r.path != p);
                }
                if clear {
                    app.execute("file.clear_recent");
                }

                ui.add_space(28.0);
                ui.horizontal(|ui| {
                    if widgets::ghost_button(ui, "settings", tl!("Preferences…")).clicked() {
                        app.execute("app.preferences");
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    ui.collapsing(tl!("Folders"), |ui| {
                        ui.set_min_width(300.0);
                        crate::folders_ui::section(app, ui);
                    });
                });
                ui.add_space(28.0);
            });
        });
    });
}
