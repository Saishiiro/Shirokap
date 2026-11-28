use eframe::egui::{self, pos2, Color32, Key, Rect, Stroke, ViewportCommand};

struct Pin {
    tex: egui::TextureHandle,
    size: egui::Vec2,
    scale: f32,
    eco: Option<bool>, // mode efficacité en cours (vrai quand la fenêtre n'a pas le focus)
}

/// Fenêtre flottante, toujours au premier plan, qui affiche une capture (référence, carte, tuto…).
/// Glisser = déplacer · molette = taille · clic droit ou Échap = fermer.
pub fn run(path: String) {
    let Ok(img) = image::open(&path) else { return };
    let img = img.to_rgba8();
    let size = egui::vec2(img.width() as f32, img.height() as f32);
    let scale = (640.0 / size.x.max(size.y)).min(1.0);
    let opts = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(size * scale)
            .with_decorations(false)
            .with_always_on_top()
            .with_title("Shirokap pin"),
        ..Default::default()
    };
    let _ = eframe::run_native(
        "Shirokap pin",
        opts,
        Box::new(move |cc| {
            let ci = egui::ColorImage::from_rgba_unmultiplied(
                [img.width() as usize, img.height() as usize],
                img.as_raw(),
            );
            let tex = cc.egui_ctx.load_texture("pin", ci, Default::default());
            Ok(Box::new(Pin { tex, size, scale, eco: None }))
        }),
    );
}

impl eframe::App for Pin {
    fn update(&mut self, ctx: &egui::Context, _f: &mut eframe::Frame) {
        // capture épinglée : elle ne fait qu'afficher une image, donc mode efficacité (feuille verte) ;
        // pleine vitesse le temps qu'on la déplace ou la redimensionne
        let foc = ctx.input(|i| i.focused);
        if self.eco != Some(!foc) {
            self.eco = Some(!foc);
            crate::common::set_background_mode(!foc);
        }
        let (pressed, scroll, esc, rclick) = ctx.input(|i| {
            (
                i.pointer.primary_pressed(),
                i.smooth_scroll_delta.y,
                i.key_pressed(Key::Escape),
                i.pointer.secondary_clicked(),
            )
        });
        if esc || rclick {
            ctx.send_viewport_cmd(ViewportCommand::Close);
            return;
        }
        if pressed {
            ctx.send_viewport_cmd(ViewportCommand::StartDrag);
        }
        if scroll != 0.0 {
            self.scale = (self.scale * (1.0 + scroll * 0.002)).clamp(0.1, 4.0);
            ctx.send_viewport_cmd(ViewportCommand::InnerSize(self.size * self.scale));
        }
        egui::CentralPanel::default().frame(egui::Frame::none()).show(ctx, |ui| {
            let r = ctx.screen_rect();
            let p = ui.painter();
            p.image(self.tex.id(), r, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
            p.rect_stroke(r, 0.0, Stroke::new(1.0_f32, Color32::from_black_alpha(160)));
        });
    }
}
