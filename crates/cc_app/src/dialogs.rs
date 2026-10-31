//! Application dialogs, all built on [`crate::dialog::Dialog`].

use crate::actions;
use crate::dialog::{Dialog, Kind, session};
use crate::state::AppState;
use crate::t;
use crate::theme::{self, PALETTE};
use egui::{Context, RichText, Ui};
use std::hash::{Hash, Hasher};

fn key(parts: impl Hash) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    parts.hash(&mut h);
    h.finish()
}

fn done(state: &mut AppState, r: &egui::Response) {
    if r.drag_stopped() || r.lost_focus() {
        state.session.doc.break_merge();
    }
}

pub fn show(ctx: &Context, state: &mut AppState) {
    calibration_distance(ctx, state);
    import_reference(ctx, state);
    references(ctx, state);
    project_info(ctx, state);
    crate::units_dialog::show(ctx, state);
    levels(ctx, state);
    crate::material_browser::show(ctx, state);
    crate::type_dialog::show(ctx, state);
    crate::component_editor::show(ctx, state);
    crate::schedule_ui::dialog(ctx, state);
    crate::systems_dialog::show(ctx, state);
    crate::systems_dialog::sizing(ctx, state);
    crate::graphics_dialog::show(ctx, state);
    crate::phases_dialog::show(ctx, state);
    confirm(ctx, state);
    error(ctx, state);
    about(ctx, state);
    update_report(ctx, state);
}

fn import_reference(ctx: &Context, state: &mut AppState) {
    let Some(mut options) = state.dialogs.reference_import.clone() else { return };
    let pdf = options.path.extension().and_then(|s| s.to_str()).is_some_and(|s| s.eq_ignore_ascii_case("pdf"));
    let dxf = options.path.extension().and_then(|s| s.to_str()).is_some_and(|s| s.eq_ignore_ascii_case("dxf"));
    let response = Dialog::new("reference-import", t!("reference-import-title")).width(440.0).show(
        ctx,
        |ui| {
            ui.label(options.path.display().to_string());
            if pdf {
                ui.horizontal(|ui| {
                    ui.label(t!("reference-page"));
                    ui.add(egui::DragValue::new(&mut options.page).range(1..=10_000));
                });
            }
            ui.horizontal(|ui| {
                ui.label(t!("reference-width"));
                ui.add(
                    egui::DragValue::new(&mut options.width)
                        .range(if dxf { 0.0..=100_000.0 } else { 0.001..=100_000.0 })
                        .speed(0.1),
                );
            });
            if dxf {
                ui.label(t!("reference-native-width"));
            }
        },
        |footer| {
            footer.primary(t!("action-import-reference"));
            footer.secondary(t!("action-cancel"));
        },
    );
    if response.clicked == Some(0)
        && let Some(view) = state.active_view
    {
        let command = cc_engine::commands::reference::ReferenceImport {
            view: cc_engine::Ref::Id(view.0),
            path: options.path.clone(),
            width: (options.width > 0.0).then_some(options.width),
            origin: None,
            page: Some(options.page),
        };
        if actions::run_command(state, command).is_some() {
            state.dialogs.reference_import = None;
            state.dialogs.references = true;
            state.view2d.entry(view).or_default().request_fit();
            return;
        }
    }
    if response.clicked == Some(1) || response.dismissed {
        state.dialogs.reference_import = None;
    } else {
        state.dialogs.reference_import = Some(options);
    }
}

fn references(ctx: &Context, state: &mut AppState) {
    if !state.dialogs.references {
        return;
    }
    let Some(id) = state.active_view else {
        state.dialogs.references = false;
        return;
    };
    let items: Vec<_> = state
        .project()
        .view(id)
        .map(|v| v.references.iter().map(|r| (r.name.clone(), r.width, r.origin, r.visible)).collect())
        .unwrap_or_default();
    let mut remove = None;
    let mut calibrate = None;
    let response = Dialog::new("references", t!("reference-manage-title")).width(480.0).show(
        ctx,
        |ui| {
            egui::ScrollArea::vertical().max_height(460.0).show(ui, |ui| {
                for (index, (name, width, origin, visible)) in items.iter().enumerate() {
                    ui.separator();
                    ui.label(name);
                    let (mut width, mut x, mut y, mut visible) = (*width, origin.x, origin.y, *visible);
                    let mut changed = false;
                    ui.horizontal(|ui| {
                        changed |= ui.checkbox(&mut visible, t!("reference-visible")).changed();
                        if ui.add_enabled(visible, egui::Button::new(t!("reference-calibrate"))).clicked() {
                            calibrate = Some(index);
                        }
                        if ui.button(t!("reference-remove")).clicked() {
                            remove = Some(index);
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label(t!("reference-width"));
                        changed |=
                            ui.add(egui::DragValue::new(&mut width).range(0.001..=100_000.0).speed(0.1)).changed();
                    });
                    ui.horizontal(|ui| {
                        ui.label(t!("reference-origin-x"));
                        changed |= ui.add(egui::DragValue::new(&mut x).speed(0.1)).changed();
                        ui.label(t!("reference-origin-y"));
                        changed |= ui.add(egui::DragValue::new(&mut y).speed(0.1)).changed();
                    });
                    if changed {
                        actions::run_command(
                            state,
                            cc_engine::commands::reference::ReferenceSet {
                                view: cc_engine::Ref::Id(id.0),
                                index,
                                width: Some(width),
                                origin: Some([x, y]),
                                visible: Some(visible),
                            },
                        );
                    }
                }
            });
        },
        |footer| {
            footer.primary(t!("action-close"));
        },
    );
    if let Some(index) = remove {
        actions::run_command(
            state,
            cc_engine::commands::reference::ReferenceRemove { view: cc_engine::Ref::Id(id.0), index },
        );
    }
    if let Some(index) = calibrate {
        state.reference_calibration =
            Some(crate::state::ReferenceCalibration { view: id, index, first: None, second: None, distance: 1.0 });
        state.dialogs.references = false;
        state.view2d.entry(id).or_default().zoom_region = false;
        state.message = t!("reference-pick-first");
        return;
    }
    if response.clicked.is_some() || response.dismissed {
        state.dialogs.references = false;
    }
}

fn calibration_distance(ctx: &Context, state: &mut AppState) {
    let Some(mut calibration) = state.reference_calibration.clone() else { return };
    let (Some(first), Some(second)) = (calibration.first, calibration.second) else { return };
    let measured = first.distance(second);
    let response = Dialog::new("reference-calibration", t!("reference-calibrate-title")).width(370.0).show(
        ctx,
        |ui| {
            ui.label(t!("reference-measured", "distance" => format!("{measured:.3}")));
            ui.label(t!("reference-real-distance"));
            ui.add(egui::DragValue::new(&mut calibration.distance).range(0.001..=100_000.0).speed(0.01).suffix(" m"));
        },
        |footer| {
            footer.primary(t!("reference-apply"));
            footer.secondary(t!("action-cancel"));
        },
    );
    if response.clicked == Some(0) {
        let command = cc_engine::commands::reference::ReferenceCalibrate {
            view: cc_engine::Ref::Id(calibration.view.0),
            index: calibration.index,
            first: first.to_array(),
            second: second.to_array(),
            distance: calibration.distance,
        };
        if actions::run_command(state, command).is_some() {
            state.reference_calibration = None;
            state.dialogs.references = true;
            return;
        }
    }
    if response.clicked == Some(1) || response.dismissed {
        state.reference_calibration = None;
    } else {
        state.reference_calibration = Some(calibration);
    }
}

/// Footer of a data dialog: OK keeps the edits, Cancel (or Esc / close) undoes them.
/// Returns true when the dialog closes.
pub(crate) fn ok_cancel(state: &mut AppState, response: &crate::dialog::DialogResponse, ok: usize) -> bool {
    match response.clicked {
        Some(i) if i == ok => {
            session::commit(state);
            true
        }
        Some(_) => {
            session::cancel(state);
            true
        }
        None if response.dismissed => {
            session::cancel(state);
            true
        }
        None => false,
    }
}

/// Label of a form row.
fn form_label(ui: &mut Ui, label: &str) {
    ui.label(RichText::new(label).color(PALETTE.text_muted));
}

fn project_info(ctx: &Context, state: &mut AppState) {
    if !state.dialogs.project_info {
        return;
    }
    session::begin(state);
    let mut ok = 0;
    let response = Dialog::new("project-info", t!("dlg-project-info")).width(440.0).show(
        ctx,
        |ui| {
            let info = state.project().info.clone();
            egui::Grid::new("info").num_columns(2).spacing([14.0, 8.0]).min_col_width(110.0).show(ui, |ui| {
                let fields = [
                    (t!("info-name"), "name", info.name),
                    (t!("info-number"), "number", info.number),
                    (t!("info-client"), "client", info.client),
                    (t!("info-address"), "address", info.address),
                    (t!("info-author"), "author", info.author),
                ];
                for (label, field, value) in fields {
                    form_label(ui, &label);
                    let mut text = value;
                    let r = ui.add(egui::TextEdit::singleline(&mut text).desired_width(300.0));
                    if r.changed() {
                        crate::engine_edit::info(state, Some(key(("info", field))), |info| match field {
                            "name" => info.name = text,
                            "number" => info.number = text,
                            "client" => info.client = text,
                            "address" => info.address = text,
                            _ => info.author = text,
                        });
                    }
                    done(state, &r);
                    ui.end_row();
                }
            });
        },
        |f| {
            ok = f.primary(t!("action-ok"));
            f.secondary(t!("action-cancel"));
        },
    );
    if ok_cancel(state, &response, ok) {
        state.dialogs.project_info = false;
    }
}

fn levels(ctx: &Context, state: &mut AppState) {
    if !state.dialogs.levels {
        return;
    }
    session::begin(state);
    let mut ok = 0;
    let response = Dialog::new("levels", t!("dlg-levels")).width(460.0).show(
        ctx,
        |ui| {
            let levels = state.project().levels.clone();
            egui::Grid::new("levels").num_columns(3).striped(true).spacing([12.0, 6.0]).show(ui, |ui| {
                form_label(ui, &t!("prop-name"));
                form_label(ui, &t!("level-elevation"));
                ui.label("");
                ui.end_row();
                for level in levels.iter().rev() {
                    let mut name = level.name.clone();
                    let r = ui.add_sized([220.0, 22.0], egui::TextEdit::singleline(&mut name));
                    if r.changed() {
                        crate::engine_edit::level(state, level.id, Some(key((level.id, "name"))), |l| l.name = name);
                    }
                    done(state, &r);
                    let mut elev = level.elevation;
                    let r = ui.add(crate::properties::length_drag(&mut elev).speed(0.01));
                    if r.changed() {
                        crate::engine_edit::level(state, level.id, Some(key((level.id, "elev"))), |l| {
                            l.elevation = elev;
                        });
                    }
                    done(state, &r);
                    if ui.add_enabled(levels.len() > 1, egui::Button::new(t!("action-delete"))).clicked() {
                        state.dialogs.delete_level = Some(level.id);
                    }
                    ui.end_row();
                }
            });
            ui.add_space(4.0);
            if ui.button(t!("level-add")).clicked() {
                actions::add_level(state);
            }
        },
        |f| {
            ok = f.primary(t!("action-ok"));
            f.secondary(t!("action-cancel"));
        },
    );
    if ok_cancel(state, &response, ok) {
        state.dialogs.levels = false;
    }
    delete_level(ctx, state);
}

/// Confirmation before deleting a level (and the views on it), shown over the levels dialog.
fn delete_level(ctx: &Context, state: &mut AppState) {
    let Some(level) = state.dialogs.delete_level else { return };
    let project = state.project();
    let name = project.level_name(level).to_string();
    let in_use = project.elements.values().any(|e| e.level == level);
    let views = project.views.values().filter(|v| v.level() == Some(level)).count();
    let mut delete = None;
    let response = Dialog::new("delete-level", t!("dlg-delete-level")).kind(Kind::Warning).width(380.0).show(
        ctx,
        |ui| {
            if in_use {
                ui.label(t!("msg-level-in-use"));
            } else {
                ui.label(t!("dlg-delete-level-text", "level" => name, "views" => views));
            }
        },
        |f| {
            if in_use {
                f.primary(t!("action-ok"));
            } else {
                delete = Some(f.danger(t!("action-delete")));
                f.secondary(t!("action-cancel"));
            }
        },
    );
    if response.clicked.is_some() && response.clicked == delete {
        actions::delete_level(state, level);
    }
    if response.clicked.is_some() || response.dismissed {
        state.dialogs.delete_level = None;
    }
}

pub(crate) fn material_combo(
    ui: &mut Ui,
    state: &AppState,
    id: impl Hash + std::fmt::Debug,
    current: &mut String,
) -> bool {
    let before = current.clone();
    egui::ComboBox::from_id_salt(id).selected_text(current.as_str()).width(140.0).show_ui(ui, |ui| {
        for m in &state.project().materials {
            ui.selectable_value(current, m.name.clone(), &m.name);
        }
    });
    *current != before
}

fn confirm(ctx: &Context, state: &mut AppState) {
    let Some(action) = state.dialogs.confirm.clone() else { return };
    let (mut save, mut dont_save) = (0, 0);
    let response = Dialog::new("confirm", t!("dlg-unsaved")).kind(Kind::Warning).width(400.0).show(
        ctx,
        |ui| {
            ui.label(t!("dlg-unsaved-text"));
        },
        |f| {
            save = f.primary(t!("action-save"));
            dont_save = f.secondary(t!("action-dont-save"));
            f.secondary(t!("action-cancel"));
        },
    );
    match response.clicked {
        Some(i) if i == save => {
            state.dialogs.confirm = None;
            if actions::save(state, false) {
                actions::run_pending(state, action);
            }
        }
        Some(i) if i == dont_save => {
            state.dialogs.confirm = None;
            actions::run_pending(state, action);
        }
        Some(_) => state.dialogs.confirm = None,
        None if response.dismissed => state.dialogs.confirm = None,
        None => {}
    }
}

fn error(ctx: &Context, state: &mut AppState) {
    let Some(msg) = state.dialogs.error.clone() else { return };
    let response = Dialog::new("error", t!("dlg-error")).kind(Kind::Error).width(400.0).show(
        ctx,
        |ui| {
            ui.label(msg);
        },
        |f| {
            f.primary(t!("action-ok"));
        },
    );
    if response.clicked.is_some() || response.dismissed {
        state.dialogs.error = None;
    }
}

fn about(ctx: &Context, state: &mut AppState) {
    if !state.dialogs.about {
        return;
    }
    let repository = env!("CARGO_PKG_REPOSITORY");
    let mut open_repo = 0;
    let mut check = 0;
    let auto = &mut state.updates.auto;
    let response = Dialog::new("about", t!("dlg-about")).width(440.0).show(
        ctx,
        |ui| {
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(egui::vec2(64.0, 64.0), egui::Sense::hover());
                if let Some(logo) = theme::logo(ui.ctx()) {
                    let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
                    ui.painter().image(logo.id(), rect, uv, egui::Color32::WHITE);
                }
                ui.add_space(8.0);
                ui.vertical(|ui| {
                    ui.add_space(8.0);
                    ui.label(RichText::new("ConstructCAD").family(theme::semibold()).size(22.0).color(PALETTE.text));
                    let version = format!("{} {}", t!("about-version"), env!("CARGO_PKG_VERSION"));
                    ui.label(RichText::new(version).color(PALETTE.text_muted));
                });
            });
            ui.add_space(8.0);
            ui.label(t!("about-text"));
            ui.label(RichText::new(t!("about-familiar")).color(PALETTE.text_muted));
            ui.add_space(6.0);
            ui.separator();
            ui.add_space(4.0);
            egui::Grid::new("about-info").num_columns(2).striped(false).spacing([16.0, 6.0]).show(ui, |ui| {
                ui.label(theme::muted(t!("about-author")));
                ui.label(env!("CARGO_PKG_AUTHORS"));
                ui.end_row();
                ui.label(theme::muted(t!("about-license-label")));
                ui.hyperlink_to("GPL-3.0-or-later", "https://www.gnu.org/licenses/gpl-3.0.html");
                ui.end_row();
                ui.label(theme::muted(t!("about-repository")));
                ui.hyperlink_to(repository.trim_start_matches("https://"), repository);
                ui.end_row();
            });
            ui.add_space(4.0);
            ui.checkbox(auto, t!("update-auto")).on_hover_text(t!("update-auto-hint"));
            ui.add_space(8.0);
            ui.label(theme::title(t!("about-credits")));
            for line in [t!("about-built-with"), t!("about-font-credit"), t!("about-icon-credit")] {
                ui.label(RichText::new(line).size(12.0).color(PALETTE.text_muted));
            }
            ui.add_space(8.0);
            ui.label(RichText::new(t!("about-independence")).size(11.5).color(PALETTE.text_faint));
        },
        |f| {
            open_repo = f.extra(t!("about-open-repository"));
            check = f.extra(t!("action-check-updates"));
            f.primary(t!("action-close"));
        },
    );
    match response.clicked {
        Some(i) if i == open_repo => ctx.open_url(egui::OpenUrl::new_tab(repository)),
        Some(i) if i == check => {
            // The report replaces the About dialog.
            state.dialogs.about = false;
            state.updates.start(ctx, true);
        }
        Some(_) => state.dialogs.about = false,
        None if response.dismissed => state.dialogs.about = false,
        None => {}
    }
}

/// The result of a check for updates the user asked for (and the wait for it).
fn update_report(ctx: &Context, state: &mut AppState) {
    use crate::update::Outcome;
    let updates = &mut state.updates;
    if !updates.checking() && updates.report.is_none() {
        return;
    }
    let report = updates.report.clone();
    let current = env!("CARGO_PKG_VERSION");
    let (mut download, mut skip) = (usize::MAX, usize::MAX);
    let mut dialog = Dialog::new("update-check", t!("action-check-updates")).width(420.0);
    if matches!(report, Some(Outcome::Failed | Outcome::Disabled)) {
        dialog = dialog.kind(Kind::Warning);
    }
    let response = dialog.show(
        ctx,
        |ui| match &report {
            None => {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(t!("update-checking"));
                });
            }
            Some(Outcome::UpToDate) => {
                ui.label(t!("update-up-to-date", "version" => current));
            }
            Some(Outcome::Available(release)) => {
                ui.label(t!("update-available", "version" => release.version.to_string()));
                ui.label(theme::muted(t!("update-current", "version" => current)));
            }
            Some(Outcome::Failed) => {
                ui.label(t!("update-failed"));
            }
            Some(Outcome::Disabled) => {
                ui.label(t!("update-disabled", "variable" => crate::update::NO_CHECK_ENV));
            }
        },
        |f| {
            if matches!(report, Some(Outcome::Available(_))) {
                download = f.primary(t!("update-download"));
                skip = f.extra(t!("update-skip"));
                f.secondary(t!("action-close"));
            } else {
                f.primary(t!("action-close"));
            }
        },
    );
    match response.clicked {
        Some(i) if i == download => {
            if let Some(Outcome::Available(release)) = &report {
                ctx.open_url(egui::OpenUrl::new_tab(&release.url));
            }
            updates.dismiss();
            updates.cancel_report();
        }
        Some(i) if i == skip => {
            updates.skip();
            updates.cancel_report();
        }
        Some(_) => updates.cancel_report(),
        None if response.dismissed => updates.cancel_report(),
        None => {}
    }
}
