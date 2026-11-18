//! The Linux desktop's global menu: Pliego's menus exported over D-Bus as a `com.canonical.dbusmenu`
//! object and registered with the desktop's `com.canonical.AppMenu.Registrar`, so a top bar that
//! shows the focused app's menus (Nodalix's) shows Pliego's. The menus are the in-window bar's
//! (`menus::menu_tree`); this file only serves them and hands the chosen commands back.
//!
//! A window on Wayland has no global id, so the registration names this process instead, and only
//! a registrar that pairs a menu with its window by process can show it: Nodalix's, whose process
//! also owns `space.unmade.GlobalMenu`. With any other registrar, none, or one that goes away, the
//! menus stay in (or come back to) the window. Keys stay with the window: the shortcuts run as
//! they do without a global menu.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use pliego_ui_egui::DesignApp;
use pliego_ui_egui::menus::{self, Item};
use serde_json::Value as Json;
use zbus::blocking::Connection;
use zbus::zvariant::{OwnedValue, StructureBuilder, Value};

const REGISTRAR: &str = "com.canonical.AppMenu.Registrar";
const REGISTRAR_PATH: &str = "/com/canonical/AppMenu/Registrar";
const NODALIX_MENU: &str = "space.unmade.GlobalMenu";
const MENU_PATH: &str = "/com/getnodalia/GlobalMenu";
const MENU_INTERFACE: &str = "com.canonical.dbusmenu";
/// How often the menus' labels, enablement and checks are read again.
const REFRESH_MS: f64 = 250.0;

/// One row of the exported menu.
#[derive(Clone, Debug, PartialEq)]
enum Row {
    Item { label: String, enabled: bool, checked: Option<bool>, command: String, params: Json },
    Separator,
    Submenu { label: String, children: Vec<i32> },
}

/// The menus as the desktop reads them: rows by id (0 is the root, whose children are the menus).
#[derive(Default)]
struct Shared {
    rows: HashMap<i32, Row>,
    root: Vec<i32>,
    revision: u32,
    /// Commands chosen in the top bar, waiting for the next frame.
    chosen: Vec<(String, Json)>,
    /// Whether the desktop's registrar took the menus (the window then doesn't draw them).
    accepted: bool,
}

/// The rows of `tree`, numbered depth first from 1: the same structure always gets the same ids.
/// `item` gives a command's label, enablement and check; `None` leaves the command out.
fn rows_of(
    tree: &[(String, Vec<Item>)],
    item: &dyn Fn(&str, &str, &Json) -> Option<(String, bool, Option<bool>)>,
    title: &dyn Fn(&str) -> String,
) -> (HashMap<i32, Row>, Vec<i32>) {
    fn level(
        entries: &[Item],
        next: &mut i32,
        rows: &mut HashMap<i32, Row>,
        item: &dyn Fn(&str, &str, &Json) -> Option<(String, bool, Option<bool>)>,
        title: &dyn Fn(&str) -> String,
    ) -> Vec<i32> {
        let mut ids = Vec::new();
        for e in entries {
            let row = match e {
                // No separator first, last or twice.
                Item::Sep if ids.last().is_none_or(|l| matches!(rows.get(l), Some(Row::Separator))) => continue,
                Item::Sep => Row::Separator,
                Item::Sub(label, children) => {
                    let id = *next;
                    *next = next.saturating_add(1);
                    let children = level(children, next, rows, item, title);
                    if children.is_empty() {
                        continue;
                    }
                    rows.insert(id, Row::Submenu { label: title(label), children });
                    ids.push(id);
                    continue;
                }
                Item::Cmd { label, id, params, .. } => {
                    let Some((label, enabled, checked)) = item(label, id, params) else { continue };
                    Row::Item { label, enabled, checked, command: id.clone(), params: params.clone() }
                }
            };
            let id = *next;
            *next = next.saturating_add(1);
            rows.insert(id, row);
            ids.push(id);
        }
        if ids.last().is_some_and(|l| matches!(rows.get(l), Some(Row::Separator)))
            && let Some(l) = ids.pop()
        {
            rows.remove(&l);
        }
        ids
    }
    let (mut rows, mut next, mut root) = (HashMap::new(), 1i32, Vec::new());
    for (name, entries) in tree {
        let id = next;
        next = next.saturating_add(1);
        let children = level(entries, &mut next, &mut rows, item, title);
        rows.insert(id, Row::Submenu { label: title(name), children });
        root.push(id);
    }
    (rows, root)
}

/// The in-window menus as `app` shows them now, with File › Open Recent filled in.
fn tree_of(app: &DesignApp) -> Vec<(String, Vec<Item>)> {
    fn fill_recent(entries: &mut [Item], path: &str, app: &DesignApp) {
        for e in entries {
            if let Item::Sub(label, children) = e {
                let path = format!("{path}/{label}");
                if path == menus::OPEN_RECENT {
                    let mut recent = menus::recent_items(app);
                    recent.append(children);
                    *children = recent;
                } else {
                    fill_recent(children, &path, app);
                }
            }
        }
    }
    let mut tree: Vec<(String, Vec<Item>)> = menus::menu_tree().into_iter().map(|(t, e)| (t.to_string(), e)).collect();
    for (title, entries) in &mut tree {
        fill_recent(entries, title, app);
    }
    tree
}

impl Shared {
    fn children(&self, id: i32) -> &[i32] {
        match self.rows.get(&id) {
            _ if id == 0 => &self.root,
            Some(Row::Submenu { children, .. }) => children,
            _ => &[],
        }
    }

    /// `id`'s dbusmenu properties.
    fn properties(&self, id: i32) -> HashMap<String, OwnedValue> {
        let mut p = HashMap::new();
        let mut put = |k: &str, v: Value<'_>| {
            if let Ok(v) = v.try_to_owned() {
                p.insert(k.to_string(), v);
            }
        };
        match self.rows.get(&id) {
            None if id == 0 => put("children-display", Value::from("submenu")),
            None => {}
            Some(Row::Separator) => put("type", Value::from("separator")),
            Some(Row::Submenu { label, .. }) => {
                put("label", Value::from(mnemonic_free(label)));
                put("children-display", Value::from("submenu"));
            }
            Some(Row::Item { label, enabled, checked, .. }) => {
                put("label", Value::from(mnemonic_free(label)));
                put("enabled", Value::from(*enabled));
                if let Some(on) = checked {
                    put("toggle-type", Value::from("checkmark"));
                    put("toggle-state", Value::from(i32::from(*on)));
                }
            }
        }
        p
    }

    /// `id` and `depth` levels of its children (`-1`: all), the reply to `GetLayout`.
    fn layout_item(&self, id: i32, depth: i32) -> LayoutItem {
        let children = if depth == 0 {
            Vec::new()
        } else {
            self.children(id).iter().filter_map(|&c| self.layout(c, depth.saturating_sub(1)).try_to_owned().ok()).collect()
        };
        LayoutItem { id, properties: self.properties(id), children }
    }

    /// [`Shared::layout_item`] as a variant's value: how dbusmenu nests the children.
    fn layout(&self, id: i32, depth: i32) -> Value<'static> {
        let children: Vec<Value<'static>> =
            if depth == 0 { Vec::new() } else { self.children(id).iter().map(|&c| self.layout(c, depth.saturating_sub(1))).collect() };
        let children = zbus::zvariant::Array::from(children);
        let built = StructureBuilder::new().add_field(id).add_field(self.properties(id)).append_field(Value::Array(children)).build();
        built.map(Value::Structure).unwrap_or_else(|_| Value::from(0))
    }

    /// A click on `id`: its command, when it is an enabled item.
    fn choose(&mut self, id: i32, kind: &str) -> bool {
        if kind != "clicked" {
            return false;
        }
        let Some(Row::Item { enabled: true, command, params, .. }) = self.rows.get(&id) else { return false };
        let chosen = (command.clone(), params.clone());
        self.chosen.push(chosen);
        true
    }
}

/// A label with its underscores doubled: dbusmenu reads a single one as a mnemonic.
fn mnemonic_free(label: &str) -> String {
    label.replace('_', "__")
}

/// One item of the menu tree as dbusmenu sends it: `(ia{sv}av)`, each child a variant holding the
/// same structure.
#[derive(Debug, serde::Serialize, zbus::zvariant::Type)]
struct LayoutItem {
    id: i32,
    properties: HashMap<String, OwnedValue>,
    children: Vec<OwnedValue>,
}

fn lock(shared: &Mutex<Shared>) -> MutexGuard<'_, Shared> {
    shared.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The `com.canonical.dbusmenu` object.
struct MenuObject {
    shared: Arc<Mutex<Shared>>,
    notify: Box<dyn Fn() + Send + Sync>,
}

impl MenuObject {
    fn chosen(&self, id: i32, kind: &str) {
        if lock(&self.shared).choose(id, kind) {
            (self.notify)();
        }
    }
}

#[zbus::interface(name = "com.canonical.dbusmenu")]
impl MenuObject {
    #[zbus(property)]
    fn version(&self) -> u32 {
        3
    }
    #[zbus(property)]
    fn text_direction(&self) -> String {
        "ltr".into()
    }
    #[zbus(property)]
    fn status(&self) -> String {
        "normal".into()
    }
    #[zbus(property)]
    fn icon_theme_path(&self) -> Vec<String> {
        Vec::new()
    }

    fn get_layout(&self, parent_id: i32, recursion_depth: i32, _property_names: Vec<String>) -> (u32, LayoutItem) {
        let s = lock(&self.shared);
        (s.revision, s.layout_item(parent_id, recursion_depth))
    }

    fn get_group_properties(&self, ids: Vec<i32>, _property_names: Vec<String>) -> Vec<(i32, HashMap<String, OwnedValue>)> {
        let s = lock(&self.shared);
        // An empty list asks for every item (the dbusmenu contract).
        let ids = if ids.is_empty() { s.rows.keys().copied().collect() } else { ids };
        ids.into_iter().filter(|id| s.rows.contains_key(id)).map(|id| (id, s.properties(id))).collect()
    }

    fn get_property(&self, id: i32, name: String) -> OwnedValue {
        lock(&self.shared).properties(id).remove(&name).unwrap_or_else(|| OwnedValue::from(0i32))
    }

    fn event(&self, id: i32, event_id: String, _data: OwnedValue, _timestamp: u32) {
        self.chosen(id, &event_id);
    }

    fn event_group(&self, events: Vec<(i32, String, OwnedValue, u32)>) -> Vec<i32> {
        for (id, kind, _, _) in events {
            self.chosen(id, &kind);
        }
        Vec::new()
    }

    fn about_to_show(&self, _id: i32) -> bool {
        false
    }

    fn about_to_show_group(&self, ids: Vec<i32>) -> (Vec<i32>, Vec<i32>) {
        let s = lock(&self.shared);
        let unknown = ids.into_iter().filter(|id| *id != 0 && !s.rows.contains_key(id)).collect();
        (Vec::new(), unknown)
    }
}

/// The exported menus, kept in step with the app from its frames.
pub struct GlobalMenu {
    conn: Connection,
    shared: Arc<Mutex<Shared>>,
    last_refresh: f64,
}

impl GlobalMenu {
    /// Runs the commands chosen in the top bar, tells the window whether to draw its own menus,
    /// and every [`REFRESH_MS`] sends the desktop the menus as they are now.
    pub fn poll(&mut self, app: &mut DesignApp, ctx: &egui::Context) {
        let (chosen, accepted) = {
            let mut s = lock(&self.shared);
            (std::mem::take(&mut s.chosen), s.accepted)
        };
        let mut chosen = chosen.into_iter();
        app.poll_native_actions(ctx, || chosen.next());
        app.global_menu = accepted;
        let now = pliego_ui_egui::now_ms();
        if !accepted || now - self.last_refresh < REFRESH_MS {
            return;
        }
        self.last_refresh = now;
        let (rows, root) = current_rows(app, ctx);
        let revision = {
            let mut s = lock(&self.shared);
            if s.rows == rows && s.root == root {
                return;
            }
            s.rows = rows;
            s.root = root;
            s.revision = s.revision.wrapping_add(1);
            s.revision
        };
        // The desktop reads the menus again when told they changed; a failure leaves it on the
        // last ones it read.
        let _ = self.conn.emit_signal(None::<&str>, MENU_PATH, MENU_INTERFACE, "LayoutUpdated", &(revision, 0i32));
    }
}

/// The rows for `app`'s menus now: translated labels, enablement and checks.
fn current_rows(app: &DesignApp, ctx: &egui::Context) -> (HashMap<i32, Row>, Vec<i32>) {
    let lang = app.ui.language.clone();
    let title = |label: &str| menus::native_label(pliego_ui_egui::i18n::tr(&lang, label));
    let item = |label: &str, id: &str, params: &Json| {
        if !menus::shown(app, id) {
            return None;
        }
        let label = if id == "app.openRecent" { label.to_string() } else { title(label) };
        Some((label, menus::native_menu_enabled(app, ctx, id), menus::checked(app, id, params)))
    };
    rows_of(&tree_of(app), &item, &title)
}

/// Whether the registrar on the bus is Nodalix's, the one registrar that finds a Wayland
/// window's menu by its process.
fn nodalix_registrar(conn: &Connection) -> bool {
    let Ok(dbus) = zbus::blocking::fdo::DBusProxy::new(conn) else { return false };
    let owner = |name: &str| zbus::names::BusName::try_from(name).ok().and_then(|n| dbus.get_name_owner(n).ok()).map(|o| o.to_string());
    matches!((owner(REGISTRAR), owner(NODALIX_MENU)), (Some(r), Some(n)) if r == n)
}

/// Hand the menu to the registrar, and record whether it took it.
fn register(conn: &Connection, shared: &Mutex<Shared>, ctx: &egui::Context) {
    let path = zbus::zvariant::ObjectPath::from_static_str_unchecked(MENU_PATH);
    let accepted = nodalix_registrar(conn)
        && conn.call_method(Some(REGISTRAR), REGISTRAR_PATH, Some(REGISTRAR), "RegisterWindow", &(std::process::id(), path)).is_ok();
    lock(shared).accepted = accepted;
    ctx.request_repaint();
    log::info!("global menu: {}", if accepted { "shown in the desktop's top bar" } else { "kept in the window" });
}

/// Whether Pliego's window is a Wayland one: winit picks Wayland when the session has it, unless
/// told otherwise.
pub fn on_wayland() -> bool {
    std::env::var_os("WAYLAND_DISPLAY").is_some() && std::env::var("WINIT_UNIX_BACKEND").map_or(true, |b| b != "x11")
}

/// Export the menus and register them with the desktop's registrar, now and whenever a registrar
/// appears. `None` (the menus stay in the window) off Wayland, without a session bus, or with
/// `PLIEGO_NO_NATIVE_MENU` set.
pub fn install(app: &DesignApp, ctx: &egui::Context) -> Option<GlobalMenu> {
    if !on_wayland() || std::env::var_os("PLIEGO_NO_NATIVE_MENU").is_some() {
        return None;
    }
    let shared = Arc::new(Mutex::new(Shared { revision: 1, ..Shared::default() }));
    {
        let (rows, root) = current_rows(app, ctx);
        let mut s = lock(&shared);
        s.rows = rows;
        s.root = root;
    }
    let object = MenuObject { shared: shared.clone(), notify: app.native_action_notifier(ctx.clone()) };
    let conn = match zbus::blocking::connection::Builder::session().and_then(|b| b.serve_at(MENU_PATH, object)).and_then(|b| b.build()) {
        Ok(conn) => conn,
        Err(e) => {
            log::info!("global menu: no session bus ({e}); the menus stay in the window");
            return None;
        }
    };
    register(&conn, &shared, ctx);
    // The registrar may start after Pliego, or restart with the shell: register whenever it
    // appears, and take the menus back into the window while it is gone.
    let (watch_conn, watch_shared, watch_ctx) = (conn.clone(), shared.clone(), ctx.clone());
    let spawned = std::thread::Builder::new().name("global-menu".into()).spawn(move || {
        let Ok(dbus) = zbus::blocking::fdo::DBusProxy::new(&watch_conn) else { return };
        let Ok(changes) = dbus.receive_name_owner_changed() else { return };
        for change in changes {
            let Ok(args) = change.args() else { continue };
            if args.name().as_str() != REGISTRAR && args.name().as_str() != NODALIX_MENU {
                continue;
            }
            if args.new_owner().is_some() {
                register(&watch_conn, &watch_shared, &watch_ctx);
            } else {
                lock(&watch_shared).accepted = false;
                watch_ctx.request_repaint();
            }
        }
    });
    if let Err(e) = spawned {
        log::warn!("global menu: couldn't watch the registrar ({e})");
    }
    Some(GlobalMenu { conn, shared, last_refresh: 0.0 })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn cmd(label: &str, id: &str) -> Item {
        Item::Cmd { label: label.into(), id: id.into(), params: Json::Null, shortcut: None }
    }

    fn tree() -> Vec<(String, Vec<Item>)> {
        vec![(
            "File".into(),
            vec![
                Item::Sep,
                cmd("New_Doc", "file.new"),
                Item::Sep,
                Item::Sep,
                Item::Sub("Recent".into(), vec![cmd("a", "file.open"), cmd("Soon", "file.soon")]),
                cmd("Hidden", "cjk.only"),
                Item::Sep,
            ],
        )]
    }

    fn rows() -> Shared {
        let item =
            |label: &str, id: &str, _: &Json| (id != "cjk.only").then(|| (label.to_string(), id != "file.soon", (id == "file.new").then_some(true)));
        let (rows, root) = rows_of(&tree(), &item, &|l| l.to_string());
        Shared { rows, root, ..Shared::default() }
    }

    #[test]
    fn rows_are_numbered_depth_first_without_stray_separators() {
        let s = rows();
        assert_eq!(s.children(0), &[1]);
        let file = s.children(1).to_vec();
        assert_eq!(file.len(), 3, "item, one separator, submenu: {file:?}");
        assert!(matches!(s.rows.get(&file[1]), Some(Row::Separator)));
        assert_eq!(s.children(file[2]).len(), 2);
        // Mnemonics are escaped, checks are shown, a disabled item says so.
        let p = s.properties(file[0]);
        assert_eq!(p.get("label").and_then(|v| <&str>::try_from(v).ok()), Some("New__Doc"));
        assert_eq!(p.get("toggle-type").and_then(|v| <&str>::try_from(v).ok()), Some("checkmark"));
        let soon = s.children(file[2])[1];
        assert_eq!(s.properties(soon).get("enabled").and_then(|v| bool::try_from(v).ok()), Some(false));
        // The layout carries the whole tree, typed as dbusmenu reads it.
        use zbus::zvariant::Type as _;
        assert_eq!(LayoutItem::SIGNATURE.to_string(), "(ia{sv}av)");
        assert_eq!(s.layout_item(0, -1).children.len(), 1);
        assert!(s.layout_item(0, 0).children.is_empty());
    }

    #[test]
    fn a_click_queues_its_command_and_a_disabled_item_nothing() {
        let mut s = rows();
        let file = s.children(1).to_vec();
        let soon = s.children(file[2])[1];
        assert!(s.choose(file[0], "clicked"));
        assert!(!s.choose(soon, "clicked"), "disabled");
        assert!(!s.choose(file[0], "hovered"));
        assert!(!s.choose(9999, "clicked"), "an id the desktop made up");
        assert_eq!(s.chosen, vec![("file.new".to_string(), Json::Null)]);
    }

    #[test]
    fn the_app_menus_export_with_open_recent_filled_in() {
        let mut app = DesignApp::new(pliego_engine::Session::new(), pliego_ui_egui::Services::default());
        app.session.prefs.note_recent_file("/tmp/brochure.pliego");
        let tree = tree_of(&app);
        assert!(tree.iter().any(|(t, _)| t == "File") && tree.iter().any(|(t, _)| t == "Help"));
        let ctx = egui::Context::default();
        let (rows, root) = current_rows(&app, &ctx);
        assert_eq!(root.len(), tree.len());
        let recent =
            rows.values().find(|r| matches!(r, Row::Item { command, params, .. } if command == "app.openRecent" && *params == json!({"index": 0})));
        assert!(recent.is_some(), "File › Open Recent lists the recent file");
    }
}
