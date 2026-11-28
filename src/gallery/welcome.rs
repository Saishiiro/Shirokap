//! Écran d'accueil (au premier lancement, ou via Paramètres > Système) : une carte par-dessus la
//! fenêtre, avec la langue, quelques thèmes, les raccourcis, l'icône du tray et le dossier des captures.
use super::*;

/// Thèmes proposés sur l'écran d'accueil (les autres sont dans Paramètres > Apparence).
const PICKS: [&str; 6] = ["shirokap", "dark", "midnight", "light", "sakura", "mint"];

impl App {
    pub(super) fn welcome_ui(&mut self, ctx: &egui::Context) {
        // pas par-dessus l'éditeur ; elle peut rester ouverte dans la galerie pendant qu'on est ailleurs
        let show = self.show_welcome && !(self.tab == Tab::Gallery && self.editor.is_some());
        let k = ctx.animate_bool_with_time_and_easing(egui::Id::new("welcome_k"), show, 0.3, egui::emath::easing::cubic_out);
        if k < 0.01 {
            return;
        }
        let screen = ctx.screen_rect();
        // fond assombri qui retient les clics (un cran sous la carte : un clic dessus ne la recouvre pas,
        // et les listes déroulantes de la carte restent au-dessus)
        egui::Area::new(egui::Id::new("welcome_dim")).order(egui::Order::Middle).fixed_pos(screen.min).show(ctx, |ui| {
            ui.allocate_exact_size(screen.size(), Sense::click());
            ui.painter().rect_filled(screen, 0.0, Color32::from_black_alpha((130.0 * k) as u8));
        });
        let (mut start, mut custom, mut change) = (false, false, false);
        let mut theme: Option<&'static str> = None;
        let mut lang: Option<&'static str> = None;
        let mut keys: Vec<(&'static str, String, &'static str)> = Vec::new();
        for (i, key, d) in [
            (icons::CAMERA, self.cfg.hotkey.clone(), tr("Capturer l'écran")),
            (icons::ZONE, self.cfg.hotkey_region.clone(), tr("Capturer une zone")),
            (icons::APP_WINDOW, self.cfg.hotkey_window.clone(), tr("Capturer la fenêtre active")),
            (icons::GALLERY, self.cfg.hotkey_gallery.clone(), tr("Ouvrir la galerie")),
        ] {
            if !key.is_empty() {
                keys.push((i, key, d));
            }
        }
        let w = 660.0_f32.min(screen.width() - 40.0);
        egui::Area::new(egui::Id::new("welcome_card"))
            .order(egui::Order::Foreground)
            .anchor(Align2::CENTER_CENTER, vec2(0.0, 16.0 * (1.0 - k)))
            .interactable(show)
            .show(ctx, |ui| {
                ui.multiply_opacity(k);
                let dark = ui.visuals().dark_mode;
                egui::Frame::none()
                    .fill(ui.visuals().window_fill)
                    .rounding(24.0)
                    .stroke(Stroke::new(1.0_f32, ui.visuals().widgets.noninteractive.bg_stroke.color))
                    .shadow(egui::epaint::Shadow { offset: vec2(0.0, 18.0), blur: 56.0, spread: 0.0, color: Color32::from_black_alpha(if dark { 170 } else { 70 }) })
                    // marge droite plus courte : la barre de défilement (si la fenêtre est basse) loge
                    // dans la gouttière réservée à droite du contenu, sans le chevaucher
                    .inner_margin(egui::Margin { left: 28.0, right: 12.0, top: 28.0, bottom: 28.0 })
                    .show(ui, |ui| {
                        ui.set_width(w - 40.0);
                        egui::ScrollArea::vertical().max_height(screen.height() - 120.0).auto_shrink([false, true]).show(ui, |ui| {
                            ui.set_max_width(ui.available_width() - 16.0);
                            self.welcome_body(ui, &keys, &mut start, &mut custom, &mut change, &mut theme, &mut lang);
                        });
                    });
            });
        if !show {
            return;
        }
        if ctx.input(|i| i.key_pressed(Key::Escape)) && self.cmd.is_none() {
            start = true;
        }
        if let Some(t) = theme {
            self.save_one(|c| c.theme = t.to_string());
        }
        if let Some(l) = lang {
            self.save_one(|c| c.language = l.to_string());
        }
        if change {
            if let Some(p) = rfd::FileDialog::new().pick_folder() {
                let dir = p.display().to_string();
                self.save_one(|c| c.save_dir = dir.clone());
                self.items = list_items(&self.cfg.save_dir);
            }
        }
        if custom {
            self.tab = Tab::Settings;
            self.set_cat = SetCat::Shortcuts;
        }
        if start || custom {
            self.show_welcome = false;
            self.save_state();
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn welcome_body(
        &mut self,
        ui: &mut egui::Ui,
        keys: &[(&'static str, String, &'static str)],
        start: &mut bool,
        custom: &mut bool,
        change: &mut bool,
        theme: &mut Option<&'static str>,
        lang: &mut Option<&'static str>,
    ) {
        let v = ui.visuals().clone();
        let weak = v.weak_text_color();
        let inset = v.extreme_bg_color;
        let full = ui.available_width();
        ui.spacing_mut().item_spacing.y = 8.0;
        // en-tête : logo, titre et phrase d'accroche ; la langue à droite
        ui.horizontal(|ui| {
            big_logo(ui, 52.0);
            ui.add_space(6.0);
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 2.0;
                ui.add_space(2.0);
                ui.label(bold_text(tr("Bienvenue dans Shirokap"), 24.0));
                ui.label(egui::RichText::new(tr("Un outil de capture d'écran léger, qui tourne discrètement dans la zone de notification.")).color(weak));
            });
        });
        ui.add_space(14.0);
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(ic(icons::LANGUAGE, tr("Langue"))).color(weak));
            let name = |code: &str| -> String {
                match LANGUAGES.iter().find(|l| l.0 == code) {
                    Some(("auto", _, _)) | None => tr("Automatique (langue de Windows)").to_string(),
                    Some((_, n, true)) => n.to_string(),
                    Some((_, n, false)) => format!("{n} · {}", tr("traduction non vérifiée")),
                }
            };
            egui::ComboBox::new("welcome_lang", "").width(260.0).selected_text(name(&self.cfg.language)).show_ui(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 2.0;
                for (code, _, _) in LANGUAGES {
                    if menu_item(ui, "", &name(code), self.cfg.language == code, "").clicked() {
                        *lang = Some(code);
                    }
                }
            });
        });

        // thème : quelques vignettes, un clic l'applique tout de suite
        section(ui, tr("Choisis ton thème"));
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
            for key in PICKS {
                let Some((_, name)) = THEMES.iter().find(|t| t.0 == key) else { continue };
                let mut c = self.cfg.clone();
                c.theme = key.to_string();
                if mini_theme(ui, self.cfg.theme == key, tr(name), &palette(key), accent(&c)).clicked() {
                    *theme = Some(key);
                }
            }
        });
        ui.label(egui::RichText::new(tr("Les autres thèmes et la couleur d'accent sont dans Paramètres › Apparence.")).size(12.5).color(weak));

        // raccourcis : sur deux colonnes si la place le permet
        section(ui, tr("Tes raccourcis"));
        let cols = if full > 520.0 { 2 } else { 1 };
        let gap = 8.0;
        let cw = ((full - gap * (cols as f32 - 1.0)) / cols as f32).floor();
        for pair in keys.chunks(cols) {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = gap;
                for (icon, key, what) in pair {
                    let (r, _) = ui.allocate_exact_size(vec2(cw, 44.0), Sense::hover());
                    let p = ui.painter();
                    p.rect_filled(r, 12.0, inset);
                    let tile = Rect::from_center_size(pos2(r.min.x + 24.0, r.center().y), vec2(28.0, 28.0));
                    let acc = self.shown_acc;
                    p.rect_filled(tile, 8.0, Color32::from_rgba_unmultiplied(acc.r(), acc.g(), acc.b(), if v.dark_mode { 46 } else { 56 }));
                    p.text(tile.center(), Align2::CENTER_CENTER, *icon, FontId::proportional(15.0), v.text_color());
                    let chip = kbd_chip(ui, r.right_center() - vec2(10.0, 0.0), key);
                    let mut job = egui::text::LayoutJob::simple_singleline(what.to_string(), FontId::proportional(14.0), v.text_color());
                    job.wrap = egui::text::TextWrapping { max_width: (chip.min.x - tile.max.x - 20.0).max(30.0), max_rows: 1, break_anywhere: true, overflow_character: Some('…') };
                    let g = ui.painter().layout_job(job);
                    ui.painter().galley(pos2(tile.max.x + 10.0, r.center().y - g.size().y / 2.0), g, v.text_color());
                }
            });
        }

        // l'icône du tray : trois gestes à connaître
        section(ui, tr("Dans la zone de notification"));
        let n = if full > 460.0 { 3 } else { 1 };
        let tw = ((full - gap * (n as f32 - 1.0)) / n as f32).floor();
        let gestures = [
            (icons::GALLERY, tr("Clic gauche"), tr("Ouvrir la galerie")),
            (icons::CAMERA, tr("Clic molette"), tr("Capturer l'écran")),
            (icons::MENU, tr("Clic droit"), tr("Ouvrir le menu")),
        ];
        for chunk in gestures.chunks(n) {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = gap;
                for (icon, how, what) in chunk {
                    let (r, _) = ui.allocate_exact_size(vec2(tw, 54.0), Sense::hover());
                    let p = ui.painter();
                    p.rect_filled(r, 12.0, inset);
                    p.text(pos2(r.min.x + 16.0, r.center().y), Align2::LEFT_CENTER, *icon, FontId::proportional(17.0), weak);
                    p.text(pos2(r.min.x + 44.0, r.center().y - 9.0), Align2::LEFT_CENTER, *how, FontId::new(13.5, egui::FontFamily::Name("bold".into())), v.text_color());
                    p.text(pos2(r.min.x + 44.0, r.center().y + 10.0), Align2::LEFT_CENTER, *what, FontId::proportional(12.5), weak);
                }
            });
        }

        // dossier des captures
        section(ui, tr("Tes captures"));
        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if pill(ui, icons::FOLDER_OPEN, tr("Changer…")).clicked() {
                    *change = true;
                }
                let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 40.0), Sense::hover());
                let p = ui.painter();
                p.rect_filled(r, 12.0, inset);
                p.text(pos2(r.min.x + 14.0, r.center().y), Align2::LEFT_CENTER, icons::FOLDER, FontId::proportional(15.0), weak);
                let mut job = egui::text::LayoutJob::simple_singleline(self.cfg.save_dir.clone(), FontId::proportional(13.5), v.text_color());
                job.wrap = egui::text::TextWrapping { max_width: (r.width() - 52.0).max(30.0), max_rows: 1, break_anywhere: true, overflow_character: Some('…') };
                let g = ui.painter().layout_job(job);
                ui.painter().galley(pos2(r.min.x + 40.0, r.center().y - g.size().y / 2.0), g, v.text_color());
            });
        });

        // boutons : « C'est parti » à droite, personnaliser à gauche
        ui.add_space(22.0);
        ui.horizontal(|ui| {
            if ghost(ui, icons::KEYBOARD, tr("Personnaliser les raccourcis")).clicked() {
                *custom = true;
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if accent_pill(ui, icons::NEXT, tr("C'est parti")).clicked() {
                    *start = true;
                }
            });
        });
    }
}

/// Titre de section de l'écran d'accueil.
fn section(ui: &mut egui::Ui, title: &str) {
    ui.add_space(18.0);
    ui.label(bold_text(title, 15.0));
    ui.add_space(2.0);
}

/// Logo de l'application en grand (même image que celui du menu latéral).
fn big_logo(ui: &mut egui::Ui, size: f32) {
    let id = egui::Id::new("logo_texture_big");
    let ctx = ui.ctx().clone();
    let tex = ctx.data(|d| d.get_temp::<TextureHandle>(id)).unwrap_or_else(|| {
        let img = image::load_from_memory(include_bytes!("../../assets/logo.png"))
            .map(|i| i.resize_exact(128, 128, image::imageops::FilterType::Lanczos3).to_rgba8())
            .unwrap_or_else(|_| image::RgbaImage::new(128, 128));
        let ci = egui::ColorImage::from_rgba_unmultiplied([128, 128], img.as_raw());
        let t = ctx.load_texture("logo_big", ci, egui::TextureOptions::LINEAR);
        ctx.data_mut(|d| d.insert_temp(id, t.clone()));
        t
    });
    let (rect, _) = ui.allocate_exact_size(vec2(size, size), Sense::hover());
    ui.painter().image(tex.id(), rect, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
}

/// Petite vignette de thème : mini-fenêtre (menu et feuille de contenu) avec le nom dessous.
fn mini_theme(ui: &mut egui::Ui, selected: bool, name: &str, pal: &Pal, acc: Color32) -> egui::Response {
    let (slot, resp) = ui.allocate_exact_size(vec2(90.0, 78.0), Sense::click());
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let hk = ui.ctx().animate_bool_with_time(resp.id.with("hov"), resp.hovered(), anim::HOVER);
    let sk = ui.ctx().animate_bool_with_time(resp.id.with("sel"), selected, 0.25);
    let v = ui.visuals();
    let sel_acc = accent_of(v);
    let p = ui.painter();
    let win = Rect::from_min_size(slot.min, vec2(slot.width(), 54.0)).translate(vec2(0.0, -2.0 * hk));
    p.rect_filled(win, 10.0, pal.win);
    let side = Rect::from_min_max(win.min + vec2(4.0, 4.0), pos2(win.min.x + 24.0, win.max.y - 4.0));
    p.rect_filled(side, 6.0, pal.bg);
    p.rect_filled(Rect::from_min_size(side.min + vec2(4.0, 6.0), vec2(12.0, 5.0)), 2.5, acc);
    let sheet = Rect::from_min_max(pos2(side.max.x + 4.0, win.min.y + 4.0), win.max - vec2(4.0, 4.0));
    p.rect_filled(sheet, 6.0, pal.bg);
    let card = Rect::from_min_max(sheet.min + vec2(5.0, 5.0), pos2(sheet.max.x - 5.0, sheet.min.y + 26.0));
    p.rect_filled(card, 4.0, pal.card);
    p.rect_filled(Rect::from_min_size(card.min + vec2(5.0, 5.0), vec2(22.0, 5.0)), 2.5, acc);
    p.rect_filled(Rect::from_min_size(card.min + vec2(5.0, 13.0), vec2(card.width() - 10.0, 3.0)), 1.5, pal.line);
    let stroke_col = v.widgets.noninteractive.bg_stroke.color.lerp_to_gamma(sel_acc, sk);
    p.rect_stroke(win, 10.0, Stroke::new(1.0 + 1.5 * sk, stroke_col));
    if sk > 0.01 {
        let c = win.right_top() + vec2(-10.0, 10.0);
        p.circle_filled(c, 7.5 * sk, sel_acc);
        if sk > 0.5 {
            icon_check(p, c, on_accent(sel_acc));
        }
    }
    let font = if selected { FontId::new(12.5, egui::FontFamily::Name("bold".into())) } else { FontId::proportional(12.5) };
    p.text(pos2(slot.center().x, slot.max.y - 9.0), Align2::CENTER_CENTER, name, font, if selected { v.text_color() } else { v.weak_text_color() });
    resp
}
