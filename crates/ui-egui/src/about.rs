//! Help › About Pliego, as every Nodalia app shows it (`nodalia_theme::about`): the heading with
//! its round close button, then the page with Pliego's mark, name and version, what it is, the suite
//! it belongs to and the links. There are no Contributors or Models tabs; the authors of the code
//! this app began from are credited in `NOTICE`.

use egui::{Rect, pos2};
use nodalia_theme::about as ndl;
use pliego_engine::links;
use serde_json::json;

use crate::DesignApp;
use crate::theme::Tokens;

/// The page's links: the label (a catalog key), and the `help.<key>` command that opens it.
const LINKS: [(&str, &str); 4] = [("Pliego page", "appPage"), ("GitHub", "github"), ("getnodalia.com", "website"), ("Report an issue", "issues")];

pub fn show(app: &mut DesignApp, ctx: &egui::Context) {
    if !app.ui.about {
        return;
    }
    let t = Tokens::get(ctx);
    let look = t.creator();
    let lang = app.ui.language.clone();
    let mut open: Option<&'static str> = None;
    let mut close = false;
    let area = egui::Id::unique("about");
    let instance = area.with("placement");
    let resp = crate::modal_drag::modal(ctx, area, instance).show(ctx, |ui| {
        ui.set_width(ndl::measures::WIDTH);
        // The heading is the title row. The close button registers after the drag area, so it stays clickable.
        let margin = egui::Frame::popup(ui.style()).inner_margin;
        let top = ui.cursor().top();
        let title_row = Rect::from_min_max(
            pos2(ui.max_rect().left() - margin.leftf(), top - margin.topf()),
            pos2(ui.max_rect().right() + margin.rightf(), top + nodalia_theme::creator::layout::CLOSE),
        );
        crate::modal_drag::title_drag(ui, instance, title_row);
        egui::Frame::new().inner_margin(egui::Margin::symmetric(10, 4)).show(ui, |ui| {
            let title = crate::i18n::tr(&lang, "About Pliego");
            close = ndl::heading(ui, &look, title, egui::Id::NULL.with("pliego.about.close")).closed;
            let version = format!("{} {}", crate::i18n::tr(&lang, "Version"), env!("CARGO_PKG_VERSION"));
            let links: Vec<ndl::Link> = LINKS
                .iter()
                .map(|(label, key)| ndl::Link { label: crate::i18n::tr(&lang, label), url: links::get(key).unwrap_or_default() })
                .collect();
            let info = ndl::Info {
                name: "Pliego",
                version: &version,
                tagline: crate::i18n::tr(&lang, "Page layout for print and screen — fast, open and scriptable."),
                suite: crate::i18n::tr(&lang, "Part of the Nodalia suite of open creative tools."),
                links: &links,
                footer: crate::i18n::tr(&lang, "Open source. No telemetry."),
            };
            let page = ndl::page(ui, &look, &info, |ui, rect| crate::chrome::paint_mark(ui.painter(), rect, &t));
            if let Some(i) = page.picked
                && let Some((_, key)) = LINKS.get(i)
            {
                open = Some(key);
            }
            ui.add_space(4.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new(crate::i18n::tr(&lang, "Includes work by other open-source authors (see NOTICE).")).small().color(t.text_dim),
                );
            });
        });
    });
    if close || resp.should_close() {
        app.ui.about = false;
    }
    if let Some(k) = open {
        let _ = app.run(&format!("help.{k}"), json!({}));
    }
}
