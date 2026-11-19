//! The application bar in the Nodalia Design Language: the window's frame above the islands. A
//! hamburger with the app's own entries, Home and the work menus at the left; search and the
//! creator suite's workspace selector at the right; and, where Pliego draws the window's frame
//! (Linux), Nodalia's round window controls at the far right. Empty space moves the window and a
//! double-click there maximizes it.

use super::{DesignApp, MAC_APP_BAR, MAC_CONTENT_LEFT, Sense, Tokens, json};
use nodalia_theme::titlebar::{Placement, Side};

/// The bar's height: room for its 28 pt controls with a margin above and below them.
pub const NDL_APP_BAR: f32 = 44.0;

/// The bar's height on this platform: macOS keeps the title-bar band its traffic lights sit in.
pub fn height(mac: bool) -> f32 {
    if mac { MAC_APP_BAR } else { NDL_APP_BAR }
}

/// The hamburger's entries: (icon, label, command). Quit comes after a line.
const MAIN_MENU: [(&str, &str, &str); 4] = [
    ("info", "About Pliego", "help.about"),
    ("settings", "Preferences…", "app.preferences"),
    ("keyboard", "Keyboard Shortcuts…", "app.keyboardShortcuts"),
    ("log-out", "Quit Pliego", "app.quit"),
];

/// Nodalia's main menu: a 28 pt hamburger with the app's own entries, each with its icon and
/// shortcut. The work menus stay beside it.
fn main_menu(app: &mut DesignApp, ui: &mut egui::Ui, size: f32) {
    let t = Tokens::get(ui.ctx());
    let lang = app.ui.language.clone();
    let button = crate::icons::button(ui, "menu", size, false, crate::i18n::tr(&lang, "Main Menu"));
    let mut chosen = None;
    egui::Popup::menu(&button).show(|ui| {
        ui.set_min_width(220.0);
        for (i, (icon, label, id)) in MAIN_MENU.iter().enumerate() {
            if i == 3 {
                ui.separator();
            }
            let shortcut = crate::menus::shortcut_of(app, id).map(|sc| crate::menus::shortcut_text(&sc)).unwrap_or_default();
            let row = ui
                .horizontal(|ui| {
                    let (glyph, _) = ui.allocate_exact_size(egui::vec2(16.0, 16.0), Sense::hover());
                    if let Some(i) = nodalia_theme::icons::get(icon) {
                        nodalia_theme::icons::paint(ui.painter(), glyph, i, nodalia_theme::icons::Style::bold(t.icon));
                    }
                    ui.add(egui::Button::new(crate::rtl::widget(ui, crate::i18n::tr(&lang, label))).frame(false).shortcut_text(shortcut))
                })
                .inner;
            if row.clicked() {
                chosen = Some(*id);
                ui.close();
            }
        }
    });
    if let Some(id) = chosen {
        crate::menus::activate(app, id, &serde_json::Value::Null);
    }
}

/// The creator suite's workspace selector, at `width`: the built-in workspaces (translated), then
/// the user's own after a line, then Reset, New Workspace… and, on a workspace of the user's own,
/// Delete Workspace.
fn workspaces(app: &mut DesignApp, ui: &mut egui::Ui, width: f32) {
    let t = Tokens::get(ui.ctx());
    let lang = app.ui.language.clone();
    let current = app.ui.workspace.clone();
    let builtin: Vec<String> = crate::menus::WORKSPACES.iter().map(|w| w.to_string()).collect();
    let customs: Vec<String> = app.ui.custom_workspaces.iter().map(|w| w.name.clone()).collect();
    let names: Vec<String> = builtin.iter().chain(customs.iter()).cloned().collect();
    let shown: Vec<String> = names.iter().map(|w| crate::i18n::workspace_name(&lang, w).to_string()).collect();
    let shown_refs: Vec<&str> = shown.iter().map(String::as_str).collect();
    let reset = format!("{} {}", crate::i18n::tr(&lang, "Reset"), crate::i18n::workspace_name(&lang, &current));
    let mut actions = vec![reset.as_str(), crate::i18n::tr(&lang, "New Workspace…")];
    let own = customs.contains(&current);
    if own {
        actions.push(crate::i18n::tr(&lang, "Delete Workspace"));
    }
    let picked = nodalia_theme::creator::workspace_selector(
        ui,
        &t.creator(),
        "pliego",
        crate::i18n::workspace_name(&lang, &current),
        &shown_refs,
        builtin.len(),
        &actions,
        width,
    );
    match picked {
        Some(nodalia_theme::creator::WorkspacePick::Workspace(i)) => {
            if let Some(name) = names.get(i) {
                let _ = app.run("window.workspace", json!({"name": name}));
            }
        }
        Some(nodalia_theme::creator::WorkspacePick::Action(0)) => {
            let _ = app.run("window.resetWorkspace", json!({}));
        }
        Some(nodalia_theme::creator::WorkspacePick::Action(1)) => {
            let _ = app.run("window.newWorkspace", json!({}));
        }
        Some(nodalia_theme::creator::WorkspacePick::Action(2)) if own => {
            let _ = app.run("window.deleteWorkspace", json!({"name": current}));
        }
        _ => {}
    }
}

/// The empty bar under `rect` moves the window and a double-click on it maximizes. Registered
/// before the bar's widgets, which take their own clicks; a drag that starts on a widget (a menu
/// title) never moves the window.
fn drag_area(app: &mut DesignApp, ui: &egui::Ui, rect: egui::Rect) {
    let ctx = ui.ctx().clone();
    // Not focusable: Tab goes through the bar's widgets, not its background.
    let resp = ui.interact(rect, egui::Id::unique("window_title_drag"), Sense::CLICK | Sense::DRAG);
    if ctx.input(|i| i.pointer.any_pressed()) {
        let bare = ctx.viewport(|v| v.hits.click.is_some_and(|w| w.id == resp.id));
        ctx.data_mut(|d| d.insert_temp(resp.id, bare));
    }
    let bare = ctx.data(|d| d.get_temp(resp.id).unwrap_or(false));
    if resp.double_clicked() && bare {
        let on = !ctx.input(|i| i.viewport().maximized.unwrap_or(false));
        let _ = app.run("window.maximize", json!({"on": on}));
    } else if resp.drag_started_by(egui::PointerButton::Primary) && bare {
        let _ = app.run("window.drag", json!({}));
        // The window manager runs the move and on X11 and Wayland keeps the release: end the drag
        // now, or the next press would start none (emilk/egui#7959).
        ctx.stop_dragging();
        ctx.input_mut(|i| i.pointer = Default::default());
    }
    app.flush_window_commands(&ctx);
}

/// Nodalia's round window controls in `rect`, which the bar placed at the desktop's side.
fn window_controls(app: &mut DesignApp, ui: &egui::Ui, rect: egui::Rect, side: Side) {
    let t = Tokens::get(ui.ctx());
    let lang = app.ui.language.clone();
    let (maximized, focused) = ui.input(|i| (i.viewport().maximized.unwrap_or(false), i.viewport().focused.unwrap_or(true)));
    let mode = t.ndl_mode();
    let palette = crate::theme::ndl_palette(app.ui.brightness);
    let labels = nodalia_theme::window_controls::Labels {
        minimize: crate::i18n::tr(&lang, "Minimize"),
        maximize: crate::i18n::tr(&lang, "Maximize"),
        restore: crate::i18n::tr(&lang, "Restore"),
        close: crate::i18n::tr(&lang, "Close"),
    };
    use nodalia_theme::window_controls::Action;
    match nodalia_theme::window_controls::show_on(ui, rect, side, &palette, mode, focused, maximized, labels) {
        Some(Action::Minimize) => {
            let _ = app.run("window.minimize", json!({}));
        }
        Some(Action::Maximize) => {
            let _ = app.run("window.maximize", json!({"on": true}));
        }
        Some(Action::Restore) => {
            let _ = app.run("window.maximize", json!({"on": false}));
        }
        Some(Action::Close) => {
            let _ = app.run("window.close", json!({}));
        }
        None => {}
    }
    app.flush_window_commands(ui.ctx());
}

/// Room the window controls take at the bar's right end, with their gap to the window's edge.
pub const CONTROLS_W: f32 = nodalia_theme::window_controls::WIDTH + 8.0;

pub(super) fn ndl_app_bar(app: &mut DesignApp, ui: &mut egui::Ui) {
    let t = Tokens::get(ui.ctx());
    let mac = app.integrated_titlebar;
    let custom = app.custom_titlebar;
    // Nodalia's controls take the end of the bar the desktop keeps its own at: the right unless
    // its button layout says left.
    let side = if custom { nodalia_theme::titlebar::system_side() } else { Side::Right };
    let controls_left = custom && side == Side::Left;
    let left = if mac {
        MAC_CONTENT_LEFT as i8
    } else if controls_left {
        0
    } else {
        8
    };
    let control = if mac { 24.0 } else { 28.0 };
    egui::Panel::top("app_bar")
        .exact_size(height(mac))
        .show_separator_line(false)
        .frame(egui::Frame::NONE.fill(t.app_bar).inner_margin(egui::Margin { left, right: 0, top: 0, bottom: 0 }))
        .show(ui, |ui| {
            let full = ui.max_rect();
            let outer = egui::Rect::from_min_max(egui::pos2(full.min.x - f32::from(left), full.min.y), full.max);
            if mac {
                let _ = nodalia_window::title_bar::empty_area(ui, outer);
            }
            if custom {
                drag_area(app, ui, outer);
                window_controls(app, ui, Placement::nodalia(side).controls_rect(outer), side);
            }
            let right_edge = full.max.x - if custom && !controls_left { CONTROLS_W + 8.0 } else { 10.0 };
            let mut menus_end = full.min.x;
            let mut tools_start = right_edge;
            ui.horizontal_centered(|ui| {
                if controls_left {
                    ui.add_space(CONTROLS_W + nodalia_theme::titlebar::CONTENT_GAP);
                }
                main_menu(app, ui, control);
                if crate::icons::button(ui, "home", control, app.session.active().is_none(), crate::i18n::tr(&app.ui.language, "Home")).clicked() {
                    app.session_home();
                }
                ui.add_space(4.0);
                if !app.native_menu && !app.global_menu {
                    ui.style_mut().visuals.widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
                    ui.style_mut().visuals.widgets.inactive.bg_stroke = egui::Stroke::NONE;
                    ui.style_mut().visuals.widgets.hovered.bg_stroke = egui::Stroke::NONE;
                    ui.style_mut().spacing.button_padding = egui::vec2(8.0, 5.0);
                    crate::menus::menu_bar(app, ui);
                }
                menus_end = ui.min_rect().max.x;
                // Search and the workspace selector fill the room after the menus from the right;
                // in a narrow window the selector narrows, then search goes (the palette keeps its
                // shortcut).
                let room = (right_edge - menus_end - 12.0).max(0.0);
                let selector = room >= 96.0;
                let selector_w = room.clamp(96.0, nodalia_theme::creator::layout::WORKSPACE_WIDTH.max(66.0) + 30.0);
                let search = selector && room >= selector_w + 8.0 + super::app_bar::NDL_SEARCH_W;
                let right = egui::Rect::from_min_max(egui::pos2(menus_end + 12.0, full.min.y), egui::pos2(right_edge, full.max.y));
                let mut rui = ui.new_child(egui::UiBuilder::new().max_rect(right).layout(egui::Layout::right_to_left(egui::Align::Center)));
                if selector {
                    workspaces(app, &mut rui, selector_w);
                    rui.add_space(8.0);
                }
                if search {
                    super::app_bar::search_field(app, &mut rui);
                }
                tools_start = rui.min_rect().min.x;
            });
            let title = match app.session.active() {
                Some(d) => format!("{} — Pliego", crate::rtl::isolate(&d.title())),
                None => "Pliego".to_string(),
            };
            let galley = crate::rtl::plain(ui.ctx(), &title, super::semibold(13.0), t.text);
            let w = galley.size().x;
            let cx = outer.center().x;
            if cx - w / 2.0 > menus_end + 16.0 && cx + w / 2.0 < tools_start - 16.0 {
                ui.painter().galley(egui::pos2(cx - w / 2.0, full.center().y - galley.size().y / 2.0), galley, t.text);
            }
        });
}
