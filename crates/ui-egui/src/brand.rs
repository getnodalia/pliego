//! Pliego's own mark: the app icon (`assets/app-icon/`, Benjamin Franklin), decoded once and kept
//! as a texture, for the Home screen and About.

const ICON: &[u8] = include_bytes!("../../../assets/app-icon/hicolor/256x256/apps/dev.danielmiguelt.Pliego.png");

/// The app icon as a texture; `None` if it doesn't decode.
pub fn mark(ctx: &egui::Context) -> Option<egui::TextureHandle> {
    let id = egui::Id::unique("pliego-mark");
    if let Some(t) = ctx.data(|d| d.get_temp::<egui::TextureHandle>(id)) {
        return Some(t);
    }
    let img = pliego_images::decode_rgba(ICON)?;
    let color = egui::ColorImage::from_rgba_unmultiplied([img.width() as usize, img.height() as usize], img.as_raw());
    let tex = crate::widgets::load_texture(ctx, "pliego-mark", color, egui::TextureOptions::LINEAR);
    ctx.data_mut(|d| d.insert_temp(id, tex.clone()));
    Some(tex)
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_icon_decodes() {
        let ctx = egui::Context::default();
        let tex = super::mark(&ctx).expect("the app icon decodes");
        assert_eq!(tex.size(), [256, 256]);
    }
}
