use super::{Color32, Content, DesignApp, Sense, Stroke, Tokens, json, semibold, vec2};

pub fn start_screen(app: &mut DesignApp, ui: &mut egui::Ui) {
    let t = Tokens::get(ui.ctx());
    let r = ui.available_rect_before_wrap();
    if t.nodalia {
        ndl_home(app, ui, r);
        return;
    }
    ui.painter().rect_filled(r, 0.0, t.panel_darker);
    ui.scope_builder(
        egui::UiBuilder::new().max_rect(r.shrink(48.0)).layout(egui::Layout::top_down(if crate::i18n::is_rtl(&app.ui.language) {
            egui::Align::Max
        } else {
            egui::Align::Min
        })),
        |ui| {
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                ui.label(crate::rtl::widget(
                    ui,
                    egui::RichText::new(crate::i18n::tr(&app.ui.language, "Welcome to Pliego")).font(semibold(28.0)).color(t.text_strong),
                ));
                ui.label(crate::rtl::widget(
                    ui,
                    egui::RichText::new(crate::i18n::tr(&app.ui.language, "Page layout for print and screen — fast, open and scriptable."))
                        .size(15.0)
                        .color(t.text_dim),
                ));
                ui.add_space(24.0);
                ui.horizontal(|ui| {
                    if ui
                        .add(
                            egui::Button::new(crate::rtl::widget(
                                ui,
                                egui::RichText::new(crate::i18n::tr(&app.ui.language, "  New file  ")).size(14.0).color(Color32::WHITE),
                            ))
                            .fill(t.accent_strong)
                            .corner_radius(16.0)
                            .min_size(vec2(0.0, 32.0)),
                        )
                        .clicked()
                    {
                        let _ = app.run("app.newDocumentDialog", json!({}));
                    }
                    if ui
                        .add(
                            egui::Button::new(crate::rtl::widget(ui, egui::RichText::new(crate::i18n::tr(&app.ui.language, "  Open  ")).size(14.0)))
                                .corner_radius(16.0)
                                .min_size(vec2(0.0, 32.0)),
                        )
                        .clicked()
                    {
                        let _ = app.run("app.openDialog", json!({}));
                    }
                    if ui
                        .add(
                            egui::Button::new(crate::rtl::widget(
                                ui,
                                egui::RichText::new(crate::i18n::tr(&app.ui.language, "  Open sample magazine  ")).size(14.0),
                            ))
                            .corner_radius(16.0)
                            .min_size(vec2(0.0, 32.0)),
                        )
                        .clicked()
                    {
                        let _ = app.run("file.newSample", json!({}));
                    }
                });
                recent_files(app, ui);
                ui.add_space(24.0);
                ui.label(crate::rtl::widget(
                    ui,
                    egui::RichText::new(crate::i18n::tr(&app.ui.language, "Start a new document")).font(semibold(15.0)).color(t.text_strong),
                ));
                ui.add_space(8.0);
                ui.horizontal_wrapped(|ui| {
                    for p in pliego_doc::build::PRESETS.iter().filter(|p| p.intent == pliego_doc::Intent::Print) {
                        let (cr, resp) = ui.allocate_exact_size(vec2(120.0, 150.0), Sense::click());
                        let hov = resp.hovered();
                        ui.painter().rect_filled(cr, 6.0, if hov { t.hover } else { t.panel });
                        let k = (80.0 / p.width.max(p.height)) as f32;
                        let pr = egui::Rect::from_center_size(cr.center() - vec2(0.0, 14.0), vec2(p.width as f32 * k, p.height as f32 * k));
                        ui.painter().rect_filled(pr, 0.0, Color32::from_gray(245));
                        ui.painter().text(
                            egui::pos2(cr.center().x, cr.max.y - 26.0),
                            egui::Align2::CENTER_CENTER,
                            crate::i18n::tr(&app.ui.language, p.name),
                            semibold(12.0),
                            t.text_strong,
                        );
                        ui.painter().text(
                            egui::pos2(cr.center().x, cr.max.y - 11.0),
                            egui::Align2::CENTER_CENTER,
                            format!("{} × {}", pliego_geom::format_measure(p.width, p.units), pliego_geom::format_measure(p.height, p.units)),
                            egui::FontId::proportional(10.0),
                            t.text_dim,
                        );
                        if resp.clicked() {
                            let _ = app.run("file.new", json!({"preset": p.name}));
                        }
                    }
                });
            });
        },
    );
    let _ = Content::Unassigned;
}

/// Recent files shown on the start screen (File › Open Recent has them all).
const START_SCREEN_RECENT: usize = 8;

/// Start screen: the most recent files as cards with the first page's picture (saved in the file)
/// and the name; the full path is the tooltip. A file that is gone is dimmed and says so when
/// clicked. Nothing when the list is empty.
fn recent_files(app: &mut DesignApp, ui: &mut egui::Ui) {
    if app.session.prefs.recent_files.is_empty() {
        return;
    }
    let t = Tokens::get(ui.ctx());
    ui.add_space(24.0);
    ui.label(crate::rtl::widget(
        ui,
        egui::RichText::new(crate::i18n::tr(&app.ui.language, "Recent files")).font(semibold(15.0)).color(t.text_strong),
    ));
    ui.add_space(8.0);
    let rows: Vec<(usize, String)> = app.session.prefs.recent_files.iter().take(START_SCREEN_RECENT).cloned().enumerate().collect();
    let mut open = None;
    ui.horizontal_wrapped(|ui| {
        for (i, path) in rows {
            let stat = recent_file_stat(ui.ctx(), &path);
            let there = stat.exists;
            let modified = there.then_some(stat.modified);
            let (cr, resp) = ui.allocate_exact_size(vec2(140.0, 170.0), Sense::click());
            let resp = resp.on_hover_text(&path);
            ui.painter().rect_filled(cr, 6.0, if resp.hovered() { t.hover } else { t.panel });
            let area = egui::Rect::from_min_size(cr.min + vec2(20.0, 12.0), vec2(100.0, 108.0));
            match modified.and_then(|m| recent_preview_texture(ui.ctx(), &path, m)) {
                Some(tex) => {
                    let size = tex.size_vec2();
                    let k = (area.width() / size.x).min(area.height() / size.y);
                    let pr = egui::Rect::from_center_size(area.center(), size * k);
                    ui.painter().image(tex.id(), pr, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), Color32::WHITE);
                    ui.painter().rect_stroke(pr, 0.0, Stroke::new(1.0, t.border), egui::StrokeKind::Outside);
                }
                // No picture (saved before files had one) or the file is gone: a blank page.
                None => {
                    let pr = egui::Rect::from_center_size(area.center(), vec2(62.0, 80.0));
                    ui.painter().rect_filled(pr, 0.0, if there { Color32::from_gray(245) } else { Color32::from_gray(110) });
                }
            }
            let color = if there { t.text_strong } else { t.text_dim };
            // The name without `.pliego`, on up to two lines (broken after a space or dash).
            let label = crate::menus::recent_label(&path);
            let name = label.strip_suffix(".pliego").unwrap_or(&label).to_string();
            let mut job = egui::text::LayoutJob::simple(name, semibold(12.0), color, cr.width() - 12.0);
            job.wrap.max_rows = 2;
            job.halign = egui::Align::Center;
            let galley = ui.painter().layout_job(job);
            ui.painter().galley(egui::pos2(cr.center().x, area.max.y + 8.0), galley, color);
            if resp.clicked() {
                open = Some(i);
            }
        }
    });
    if let Some(i) = open {
        let _ = app.run("app.openRecent", json!({"index": i}));
    }
}

/// How often a recent file is looked up on disk again, in ms: the start screen and the Open Recent
/// menu draw every frame and must not touch the file system each time.
pub(super) const RECENT_STAT_MS: f64 = 3000.0;

/// A recent file as last seen on disk.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RecentStat {
    pub exists: bool,
    pub modified: Option<std::time::SystemTime>,
    checked: f64,
}

/// Whether recent file `path` is there and when it last changed, looked up again at most every
/// [`RECENT_STAT_MS`].
pub(crate) fn recent_file_stat(ctx: &egui::Context, path: &str) -> RecentStat {
    recent_file_stat_at(ctx, path, crate::now_ms())
}

pub(super) fn recent_file_stat_at(ctx: &egui::Context, path: &str, now: f64) -> RecentStat {
    let key = egui::Id::unique(("recent_stat", path));
    if let Some(st) = ctx.data(|d| d.get_temp::<RecentStat>(key))
        && (0.0..RECENT_STAT_MS).contains(&(now - st.checked))
    {
        return st;
    }
    let meta = pliego_engine::io::metadata(path).ok().filter(pliego_engine::io::Metadata::is_file);
    let st = RecentStat { exists: meta.is_some(), modified: meta.and_then(|m| m.modified().ok()), checked: now };
    ctx.data_mut(|d| d.insert_temp(key, st.clone()));
    st
}

/// The texture of a recent file's saved first-page picture, decoded once per file version.
fn recent_preview_texture(ctx: &egui::Context, path: &str, modified: Option<std::time::SystemTime>) -> Option<egui::TextureHandle> {
    type Cached = (Option<std::time::SystemTime>, Option<egui::TextureHandle>);
    let key = egui::Id::unique(("recent_preview", path));
    if let Some((m, tex)) = ctx.data(|d| d.get_temp::<Cached>(key))
        && m == modified
    {
        return tex;
    }
    let tex = pliego_engine::cmd::recent_preview(path).and_then(|p| {
        let size = [usize::try_from(p.width).ok()?, usize::try_from(p.height).ok()?];
        (size[0].checked_mul(size[1])?.checked_mul(4)? == p.rgba.len()).then(|| {
            ctx.load_texture(format!("recent_preview:{path}"), egui::ColorImage::from_rgba_unmultiplied(size, &p.rgba), egui::TextureOptions::LINEAR)
        })
    });
    ctx.data_mut(|d| d.insert_temp::<Cached>(key, (modified, tex.clone())));
    tex
}

/// The Home screen in the Nodalia Design Language: the creator suite's Home
/// (`nodalia_theme::creator::home`), the same screen as Lienzo's and Pluma's. The rail holds New
/// file, Open…, Home, the sample magazine and the links; the page, Recent Files and the print
/// presets to start from.
fn ndl_home(app: &mut DesignApp, ui: &mut egui::Ui, rect: egui::Rect) {
    use nodalia_theme::creator::home;
    let t = Tokens::get(ui.ctx());
    let look = t.creator();
    let lang = app.ui.language.clone();
    let tr = |s: &'static str| crate::i18n::tr(&lang, s);
    let website = pliego_engine::links::get("website").unwrap_or_default();
    let github = pliego_engine::links::get("github").unwrap_or_default();
    let links = [home::Link { icon: "globe", label: tr("Website"), url: website }, home::Link { icon: "code", label: tr("GitHub"), url: github }];
    let rail = home::Rail { new_file: tr("New file"), open: tr("Open…"), home: tr("Home"), learn: Some(tr("Sample Magazine")), links: &links };
    let mark = |p: &egui::Painter, r: egui::Rect| paint_mark(p, r, &t);
    let mut recent_pick = None;
    let mut preset = None;
    // Home fills the canvas island, rounded like it.
    ui.painter().rect_filled(rect, 0.0, t.app_bar);
    let picked = home::view(ui, &look, rect, "Pliego", &mark, &rail, |ui| {
        recent_pick = ndl_recents(app, ui, &look);
        ui.add_space(30.0);
        home::section_heading(ui, &look, tr("Start a new document"));
        ui.add_space(12.0);
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(14.0, 14.0);
            for p in pliego_doc::build::PRESETS.iter().filter(|p| p.intent == pliego_doc::Intent::Print) {
                if preset_card(ui, &t, &lang, p).clicked() {
                    preset = Some(p.name);
                }
            }
        });
    });
    match picked {
        Some(home::Pick::NewFile) => {
            let _ = app.run("app.newDocumentDialog", json!({}));
        }
        Some(home::Pick::Open) => {
            let _ = app.run("app.openDialog", json!({}));
        }
        Some(home::Pick::Learn) => {
            let _ = app.run("file.newSample", json!({}));
        }
        Some(home::Pick::Link(0)) => {
            let _ = app.run("help.website", json!({}));
        }
        Some(home::Pick::Link(_)) => {
            let _ = app.run("help.github", json!({}));
        }
        Some(home::Pick::Home) | None => {}
    }
    match recent_pick {
        Some(home::RecentPick::Open(i)) => {
            let _ = app.run("app.openRecent", json!({"index": i}));
        }
        Some(home::RecentPick::Remove(i)) => {
            if i < app.session.prefs.recent_files.len() {
                app.session.prefs.recent_files.remove(i);
            }
        }
        Some(home::RecentPick::Clear) => app.session.prefs.recent_files.clear(),
        Some(home::RecentPick::OpenDialog) => {
            let _ = app.run("app.openDialog", json!({}));
        }
        None => {}
    }
    if let Some(name) = preset {
        let _ = app.run("file.new", json!({"preset": name}));
    }
}

/// Pliego's mark: the app icon, or, should it not decode, a folded sheet (a *pliego*) on Pliego's
/// accent drawn in code.
pub(crate) fn paint_mark(p: &egui::Painter, r: egui::Rect, t: &Tokens) {
    let s = r.width().min(r.height());
    let r = egui::Rect::from_center_size(r.center(), vec2(s, s));
    if let Some(tex) = crate::brand::mark(p.ctx()) {
        p.image(tex.id(), r, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), Color32::WHITE);
        return;
    }
    p.rect_filled(r, s * 0.24, t.accent);
    let at = |x: f32, y: f32| egui::pos2(r.min.x + x * s, r.min.y + y * s);
    let paper = Color32::WHITE;
    // The left leaf, then the right one folded towards it.
    p.add(egui::Shape::convex_polygon(vec![at(0.22, 0.26), at(0.49, 0.32), at(0.49, 0.80), at(0.22, 0.74)], paper, Stroke::NONE));
    p.add(egui::Shape::convex_polygon(
        vec![at(0.53, 0.32), at(0.78, 0.22), at(0.78, 0.70), at(0.53, 0.80)],
        paper.gamma_multiply(0.82),
        Stroke::NONE,
    ));
}

/// A print preset as a Home card: the page's shape, its name and size.
fn preset_card(ui: &mut egui::Ui, t: &Tokens, lang: &str, p: &pliego_doc::build::Preset) -> egui::Response {
    let name = crate::i18n::tr(lang, p.name);
    let (cr, resp) = ui.allocate_exact_size(vec2(132.0, 160.0), Sense::click());
    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
    ui.painter().rect_filled(cr, t.radius, if resp.hovered() { t.hover } else { t.panel_darker });
    let k = (82.0 / p.width.max(p.height)) as f32;
    let pr = egui::Rect::from_center_size(cr.center() - vec2(0.0, 16.0), vec2(p.width as f32 * k, p.height as f32 * k));
    ui.painter().rect_filled(pr, 2.0, Color32::from_gray(248));
    ui.painter().rect_stroke(pr, 2.0, Stroke::new(1.0, t.divider), egui::StrokeKind::Outside);
    ui.painter().text(egui::pos2(cr.center().x, cr.max.y - 28.0), egui::Align2::CENTER_CENTER, name, semibold(12.5), t.text_strong);
    ui.painter().text(
        egui::pos2(cr.center().x, cr.max.y - 12.0),
        egui::Align2::CENTER_CENTER,
        format!("{} × {}", pliego_geom::format_measure(p.width, p.units), pliego_geom::format_measure(p.height, p.units)),
        egui::FontId::proportional(10.5),
        t.text_dim,
    );
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, name));
    resp
}

/// Recent Files on the Home page, as cards with the first page's picture or as a list.
fn ndl_recents(app: &mut DesignApp, ui: &mut egui::Ui, look: &nodalia_theme::creator::Look) -> Option<nodalia_theme::creator::home::RecentPick> {
    use nodalia_theme::creator::home;
    let lang = app.ui.language.clone();
    let files: Vec<String> = app.session.prefs.recent_files.iter().take(START_SCREEN_RECENT).cloned().collect();
    let rows: Vec<(String, String, Option<(egui::TextureId, egui::Vec2)>)> = files
        .iter()
        .map(|path| {
            let p = std::path::Path::new(path);
            let label = crate::menus::recent_label(path);
            let name = label.strip_suffix(".pliego").unwrap_or(&label).to_string();
            let folder = p.parent().map(|f| f.to_string_lossy().into_owned()).unwrap_or_default();
            let stat = recent_file_stat(ui.ctx(), path);
            let thumb = stat.exists.then_some(stat.modified).and_then(|m| recent_preview_texture(ui.ctx(), path, m)).map(|t| (t.id(), t.size_vec2()));
            (name, folder, thumb)
        })
        .collect();
    let list: Vec<home::Recent> =
        files.iter().zip(&rows).map(|(path, (name, folder, thumb))| home::Recent { name, folder, path, thumb: *thumb }).collect();
    let labels = home::RecentLabels {
        heading: crate::i18n::tr(&lang, "Recent Files"),
        grid: crate::i18n::tr(&lang, "Grid view"),
        list: crate::i18n::tr(&lang, "List view"),
        remove: crate::i18n::tr(&lang, "Remove from Recent"),
        clear: crate::i18n::tr(&lang, "Clear Recent Files"),
        no_folder: crate::i18n::tr(&lang, "On your computer"),
        drop: crate::i18n::tr(&lang, "Drop a Pliego, IDML or PDF file anywhere to open it."),
        drop_icon: "file",
    };
    let mut grid = !app.ui.home_recent_list;
    let pick = home::recents(ui, look, &labels, &list, &mut grid);
    app.ui.home_recent_list = !grid;
    pick
}
