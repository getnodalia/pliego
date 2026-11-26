// SPDX-License-Identifier: MIT OR Apache-2.0
//! Every screen draws its text in the interface language. The dialog or panel behind every menu
//! command is drawn in Spanish; a text that is still the English key of a Spanish catalog row was
//! drawn without going through the catalog.
// Integration tests: unwrapping and panicking on failure is fine here, unlike in shipped code.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::{BTreeMap, BTreeSet};

use egui::epaint::Shape;
use egui_kittest::Harness;
use pliego_engine::Session;
use pliego_ui_egui::{DesignApp, Services, i18n, menus};
use serde_json::{Value, json};

/// Names of what the app ships or the walk makes, drawn as stored: a swatch, a style, a master,
/// a layer. They are data the person can rename, not interface text.
const DATA: &[&str] = &[
    // Swatches, styles and sets the app ships.
    "Black",
    "Cyan",
    "Yellow",
    "[None]",
    "[Basic Paragraph]",
    "Default",
    "Parent",
    // Key names, drawn as the key caps they stand for.
    "End",
    "Home",
    "Space",
    "Tab",
];

/// Texts whose draw site is still to be found. Each is a bug, never a place to add to: remove one
/// when its site goes through `tr`.
const KNOWN: &[&str] = &[];

fn harness() -> Harness<'static, DesignApp> {
    let mut app = DesignApp::new(Session::new(), Services::default());
    app.run("file.new", json!({"pages": 2, "facingPages": true, "width": 400, "height": 500, "title": "Plate"})).unwrap();
    app.run("frame.create", json!({"rect": [36, 36, 200, 200], "content": "text"})).ok();
    app.run("frame.create", json!({"rect": [220, 36, 360, 200], "content": "graphic"})).ok();
    app.ui.language = "es".into();
    let mut h = Harness::builder().with_size(egui::vec2(1500.0, 950.0)).with_accessibility_check(false).allow_missing_glyphs().build_ui_state(
        |ui, app: &mut DesignApp| {
            app.logic(&ui.ctx().clone());
            app.ui(ui);
        },
        app,
    );
    h.run_steps(4);
    h
}

fn collect_shape(shape: &Shape, out: &mut BTreeMap<String, String>) {
    match shape {
        Shape::Text(t) => {
            for line in t.galley.text().split('\n') {
                let line = line.trim();
                if !line.is_empty() {
                    out.entry(line.to_owned()).or_insert_with(|| "painted".to_owned());
                }
            }
        }
        Shape::Vec(v) => v.iter().for_each(|s| collect_shape(s, out)),
        _ => {}
    }
}

/// Everything the frame paints as text.
fn shown(h: &Harness<'_, DesignApp>) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for clipped in &h.output().shapes {
        collect_shape(&clipped.shape, &mut out);
    }
    out
}

/// Texts of the frame that are English keys of a Spanish row with another reading.
fn untranslated(frame: &BTreeMap<String, String>) -> Vec<String> {
    frame
        .iter()
        .filter(|(t, _)| !DATA.contains(&t.as_str()) && !KNOWN.contains(&t.as_str()))
        .filter(|(t, _)| i18n::tr("es", t) != t.as_str())
        .map(|(t, kind)| format!("{t:?} ({kind})"))
        .collect()
}

/// Commands that leave the app or close the document the walk draws on.
fn skipped(id: &str) -> bool {
    id.contains("quit") || id.contains("exit") || id.starts_with("file.close") || id == "app.quit"
}

/// The commands behind the menu bar, each with the parameters its entry binds.
fn menu_commands() -> Vec<(String, Value)> {
    let mut out: Vec<(String, Value)> = Vec::new();
    for (_, entries) in menus::MENUS {
        for e in *entries {
            let Some(rest) = e.strip_prefix("cmd:").or_else(|| e.strip_prefix("ui:")) else { continue };
            let mut parts = rest.splitn(3, '|');
            let id = parts.next().unwrap_or_default().to_owned();
            let params = parts.nth(1).and_then(|p| serde_json::from_str(p).ok()).unwrap_or_else(|| json!({}));
            if !skipped(&id) && !out.iter().any(|(i, p)| *i == id && *p == params) {
                out.push((id, params));
            }
        }
    }
    out
}

fn walk() -> Vec<String> {
    let mut failures: Vec<String> = Vec::new();
    let mut seen = BTreeSet::new();
    let mut h = harness();
    let mut note = |what: &str, frame: &BTreeMap<String, String>, failures: &mut Vec<String>| {
        for line in untranslated(frame) {
            if seen.insert(line.clone()) {
                failures.push(format!("{what}: {line}"));
            }
        }
    };
    note("main window", &shown(&h), &mut failures);
    for (id, params) in menu_commands() {
        menus::activate(h.state_mut(), &id, &params);
        h.run_steps(3);
        note(&format!("command {id}"), &shown(&h), &mut failures);
        for _ in 0..3 {
            if h.state().ui.dialog.is_some() {
                pliego_ui_egui::dialogs::cancel(h.state_mut());
            }
            h.key_press(egui::Key::Escape);
            h.run_steps(2);
        }
    }
    failures
}

/// A screen that draws a text the catalog translates, without going through the catalog, shows
/// English to everyone who runs Pliego in Spanish.
#[test]
fn spanish_interface_has_no_untranslated_text() {
    let failures = walk();
    assert!(failures.is_empty(), "texts drawn in English although Spanish has them:\n{}", failures.join("\n"));
}
