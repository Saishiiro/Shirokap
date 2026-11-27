// Visionneuse, comparateur et statistiques.
use super::*;
use eframe::egui::Vec2;

impl App {
    /// Image suivante du diaporama : toujours par date de capture, dans l'ordre choisi,
    /// quel que soit le tri de la galerie (le filtre et la recherche sont respectés).
    pub(super) fn slide_next(&self, vis: &[usize], cur: usize) -> usize {
        let mut order: Vec<usize> = vis.to_vec();
        order.sort_by(|&a, &b| {
            (self.items[a].modified, file_name(&self.items[a].path))
                .cmp(&(self.items[b].modified, file_name(&self.items[b].path)))
        });
        if self.slide_new_first {
            order.reverse();
        }
        let p = order.iter().position(|&i| i == cur).unwrap_or(0);
        order[(p + 1) % order.len()]
    }

    fn slide_label(&self) -> &'static str {
        if self.slide_new_first {
            tr("du plus récent au plus ancien")
        } else {
            tr("du plus ancien au plus récent")
        }
    }

    pub(super) fn stats_ui(&mut self, ui: &mut egui::Ui) {
        let total: u64 = self.items.iter().map(|i| i.size).sum();
        let today = chrono::Local::now().date_naive();
        let day_of = |t: SystemTime| chrono::DateTime::<chrono::Local>::from(t).date_naive();
        if self.items.is_empty() {
            let text = trf("Appuie sur {0} pour capturer l'écran, ou {1} pour une zone : tes captures apparaîtront ici.", &[&self.cfg.hotkey, &self.cfg.hotkey_region]);
            if empty_view(ui, self.tab_at, self.shown_acc, icons::STATS, tr("Pas encore de statistiques"), &text, &ic(icons::GALLERY, tr("Aller à la galerie"))) {
                self.tab = Tab::Gallery;
            }
            return;
        }
        let acc = self.shown_acc;
        let since = self.tab_at.elapsed().as_secs_f32();
        // hauteur visible de la page : les panneaux se partagent la place laissée sous les tuiles
        let full_h = ui.available_height();
        egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
            ui.set_max_width(ui.available_width() - 12.0);
            // les écarts entre blocs sont posés à la main (les panneaux doivent tomber pile en bas de la page)
            ui.spacing_mut().item_spacing.y = 0.0;
            let v = ui.visuals().clone();
            let inset = v.extreme_bg_color;
            let gap = 10.0;
            // --- vue d'ensemble : quatre tuiles (la dernière, double : plus ancienne et plus lourde) ---
            let w = ui.available_width();
            let small = ((w - 3.0 * gap) / (3.0 + 2.6)).floor();
            let big = w - 3.0 * small - 3.0 * gap;
            let oldest = self.items.iter().min_by_key(|i| i.modified);
            let heaviest = self.items.iter().max_by_key(|i| i.size);
            let tile_h = 118.0;
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = gap;
                let total_s = human(total);
                let (num, unit) = total_s.rsplit_once(' ').unwrap_or((total_s.as_str(), ""));
                stat_tile(ui, small, tile_h, icons::GALLERY, acc, true, tr("Captures"), &self.items.len().to_string(), "", "", since, 0);
                stat_tile(ui, small, tile_h, icons::HARD_DRIVE, col(56, 189, 248), false, tr("Espace utilisé"), num, unit, "", since, 1);
                stat_tile(ui, small, tile_h, icons::STAR, col(251, 191, 36), false, tr("Favoris"), &self.favs.len().to_string(), "", "", since, 2);
                // tuile double
                let (r, _) = ui.allocate_exact_size(vec2(big, tile_h), Sense::hover());
                let k = ease_out(((since - 0.15) / 0.5).clamp(0.0, 1.0));
                let mut p = ui.painter().clone();
                p.multiply_opacity(k);
                let r = r.translate(vec2(0.0, 10.0 * (1.0 - k)));
                p.rect_filled(r, 14.0, inset);
                let half = (r.width() - 1.0) / 2.0;
                let left = Rect::from_min_size(r.min, vec2(half, r.height()));
                let right = Rect::from_min_size(pos2(left.max.x + 1.0, r.min.y), vec2(half, r.height()));
                p.line_segment([pos2(left.max.x, r.min.y + 16.0), pos2(left.max.x, r.max.y - 16.0)], Stroke::new(1.0_f32, v.widgets.noninteractive.bg_stroke.color));
                if let Some(o) = oldest {
                    let d = day_of(o.modified);
                    // l'année n'est affichée que si ce n'est pas l'année en cours
                    let date = d.format(dfmt(chrono::Datelike::year(&d) != chrono::Datelike::year(&today))).to_string();
                    tile_content(&p, left, icons::CALENDAR, col(52, 211, 153), false, tr("Plus ancienne"), &date, "", &file_name(&o.path), &v);
                }
                if let Some(b) = heaviest {
                    let hs = human(b.size);
                    let (num, unit) = hs.rsplit_once(' ').unwrap_or((hs.as_str(), ""));
                    tile_content(&p, right, icons::LAYERS, col(251, 113, 133), false, tr("Plus lourde"), num, unit, &file_name(&b.path), &v);
                }
                if k < 1.0 {
                    ui.ctx().request_repaint();
                }
            });
            ui.add_space(gap);

            // --- données des panneaux : captures par jour et par heure, par dossier ---
            let mut per_day: HashMap<chrono::NaiveDate, usize> = HashMap::new();
            let mut per_hour = [0usize; 24];
            for i in &self.items {
                let dt = chrono::DateTime::<chrono::Local>::from(i.modified);
                *per_day.entry(dt.date_naive()).or_default() += 1;
                per_hour[chrono::Timelike::hour(&dt) as usize % 24] += 1;
            }
            let counts: Vec<(String, usize)> = (0..7)
                .rev()
                .map(|d| {
                    let day = today - chrono::Duration::days(d);
                    let label = if d == 0 { tr("Aujourd'hui").to_string() } else { day.format(dfmt(false)).to_string() };
                    (label, per_day.get(&day).copied().unwrap_or(0))
                })
                .collect();
            let mut by: HashMap<String, (usize, u64)> = HashMap::new();
            for i in &self.items {
                let name = if i.folder.is_empty() { tr("(racine)").to_string() } else { i.folder.clone() };
                let e = by.entry(name).or_default();
                e.0 += 1;
                e.1 += i.size;
            }
            let mut folders: Vec<(String, (usize, u64))> = by.into_iter().collect();
            folders.sort_by(|a, b| b.1 .0.cmp(&a.1 .0).then_with(|| a.0.cmp(&b.0)));

            // --- quatre panneaux : en grand, deux rangées qui se partagent toute la hauteur de la page ;
            // en étroit, les uns sous les autres ---
            let w = ui.available_width();
            let side = w > 760.0;
            let (cw, fw) = if side { let c = ((w - gap) * 0.6).floor(); (c, w - gap - c) } else { (w, w) };
            if side {
                let row_h = ((full_h - tile_h - 3.0 * gap - 4.0) / 2.0).floor().max(250.0);
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = gap;
                    week_panel(ui, vec2(cw, row_h), since, &counts, acc);
                    folders_panel(ui, vec2(fw, row_h), since, &folders, acc);
                });
                ui.add_space(gap);
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = gap;
                    heat_panel(ui, vec2(cw, row_h), since, &per_day, today, acc);
                    hours_panel(ui, vec2(fw, row_h), since, &per_hour, acc);
                });
            } else {
                let fold_h = (PANEL_TOP + 40.0 + FOLDER_STEP * folders.len().min(8) as f32 - 8.0).max(200.0);
                week_panel(ui, vec2(w, 280.0), since, &counts, acc);
                ui.add_space(gap);
                folders_panel(ui, vec2(w, fold_h), since, &folders, acc);
                ui.add_space(gap);
                heat_panel(ui, vec2(w, 250.0), since, &per_day, today, acc);
                ui.add_space(gap);
                hours_panel(ui, vec2(w, 240.0), since, &per_hour, acc);
            }
            ui.add_space(12.0);
        });
    }

    pub(super) fn compare_ui(&mut self, ui: &mut egui::Ui) {
        let mut close = ui.input(|i| i.key_pressed(Key::Escape));
        ui.horizontal(|ui| {
            if ui.button(ic(icons::BACK, tr("← Retour").trim_start_matches("← "))).clicked() {
                close = true;
            }
            if let Some(c) = &self.compare {
                ui.weak(trf("A (gauche) : {0}   ·   B (droite) : {1}   ·   glisse pour comparer", &[&c.na, &c.nb]));
            }
        });
        if let Some(c) = &mut self.compare {
            let (rect, resp) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
            let s = c.a.size_vec2();
            let k = (rect.width() / s.x).min(rect.height() / s.y);
            let ir = Rect::from_center_size(rect.center(), s * k);
            if let Some(pos) = resp.interact_pointer_pos() {
                if resp.dragged() || resp.clicked() {
                    c.split = ((pos.x - ir.min.x) / ir.width()).clamp(0.0, 1.0);
                }
            }
            let p = ui.painter_at(rect);
            p.image(c.a.id(), ir, uv(), Color32::WHITE);
            let x = ir.min.x + ir.width() * c.split;
            let right = Rect::from_min_max(pos2(x, ir.min.y), ir.max);
            p.image(c.b.id(), right, Rect::from_min_max(pos2(c.split, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
            p.line_segment([pos2(x, ir.min.y), pos2(x, ir.max.y)], Stroke::new(2.0_f32, Color32::WHITE));
        }
        if close {
            self.compare = None;
        }
    }

    // ----- visionneuse -----
    pub(super) fn viewer(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, vis: &[usize], v: usize) {
        let Some(pos) = vis.iter().position(|&i| i == v) else {
            self.view = None;
            return;
        };
        let path = self.items[v].path.clone();
        let (size, modified) = (self.items[v].size, self.items[v].modified);
        if self.full.as_ref().map(|f| f.0 != path).unwrap_or(true) {
            self.full = image::open(&path).ok().map(|i| (path.clone(), tex(ctx, "full", &i.to_rgba8())));
        }
        let handle = self.full.as_ref().map(|f| f.1.clone());
        let fav = self.favs.contains(&path);
        let (mut del, mut step, mut tfav, mut edit, mut ren, mut cpath) = (false, 0i32, false, false, false, false);
        let (mut annotate, mut pin, mut cfile) = (false, false, false);

        let acc = self.shown_acc;
        let danger = col(239, 68, 68);
        if !self.fullscreen {
        ui.horizontal_wrapped(|ui| {
            if ui.button(ic(icons::BACK, tr("← Retour").trim_start_matches("← "))).clicked() {
                self.view = None;
            }
            if ui.add(egui::Button::new(icons::PREV).min_size(vec2(34.0, 30.0))).on_hover_text(tr("Précédente (←)")).clicked() {
                step = -1;
            }
            if ui.add(egui::Button::new(icons::NEXT).min_size(vec2(34.0, 30.0))).on_hover_text(tr("Suivante (→)")).clicked() {
                step = 1;
            }
            ui.separator();
            if ui.button(ic(icons::COPY, tr("Copier"))).on_hover_text(tr("Copier l'image dans le presse-papiers")).clicked() {
                if let Ok(i) = image::open(&path) {
                    copy_to_clipboard(&i.to_rgba8());
                    self.status = ic(icons::COPY, tr("Image copiée dans le presse-papiers"));
                }
            }
            let annot = egui::Button::new(egui::RichText::new(ic(icons::ANNOTATE, tr("Annoter"))).color(on_accent(acc))).fill(acc);
            if ui.add(annot).on_hover_text(tr("Ouvrir l'éditeur d'annotations")).clicked() {
                annotate = true;
            }
            let star = if fav {
                egui::RichText::new(tr("★ Favori")).color(Color32::GOLD)
            } else {
                egui::RichText::new(tr("Favori"))
            };
            if ui.add(egui::Button::new(star)).on_hover_text(tr("Favori (F)")).clicked() {
                tfav = true;
            }
            let trash = egui::Button::new(egui::RichText::new(ic(icons::TRASH, tr("Corbeille"))).color(danger));
            if ui.add(trash).on_hover_text(tr("Mettre à la corbeille (Suppr)")).clicked() {
                del = true;
            }
            dropdown(ui, &ic(icons::MORE, tr("Plus…")), false, |ui| {
                if ui.button(ic(icons::RENAME, tr("Renommer"))).clicked() {
                    ren = true;
                    ui.close_menu();
                }
                if ui.button(ic(icons::PIN, tr("Épingler à l'écran"))).clicked() {
                    pin = true;
                    ui.close_menu();
                }
                if ui.button(ic(icons::EDIT_EXT, tr("Modifier dans l'éditeur externe"))).clicked() {
                    edit = true;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button(ic(icons::EXTERNAL, tr("Ouvrir avec l'application par défaut"))).clicked() {
                    let _ = Command::new("explorer").arg(&path).spawn();
                    ui.close_menu();
                }
                if ui.button(ic(icons::FOLDER_OPEN, tr("Afficher dans le dossier"))).clicked() {
                    let _ = Command::new("explorer").arg("/select,").arg(&path).spawn();
                    ui.close_menu();
                }
                if ui.button(ic(icons::FOLDER, tr("Copier le fichier"))).clicked() {
                    cfile = true;
                    ui.close_menu();
                }
                if ui.button(ic(icons::LINK, tr("Copier le chemin"))).clicked() {
                    cpath = true;
                    ui.close_menu();
                }
                ui.separator();
                let ss = egui::Button::new(if self.slideshow { ic(icons::PAUSE, tr("Arrêter le diaporama")) } else { ic(icons::PLAY, tr("Diaporama")) })
                    .shortcut_text(tr("Espace"));
                if ui.add(ss).clicked() {
                    self.slideshow = !self.slideshow;
                    self.slide_at = Instant::now();
                    if self.slideshow {
                        self.status = ic(icons::PLAY, &trf("Diaporama : {0}", &[&self.slide_label()]));
                    }
                    ui.close_menu();
                }
                ui.weak(tr("Ordre du diaporama"));
                if ui.radio(!self.slide_new_first, tr("Du plus ancien au plus récent")).clicked() {
                    self.slide_new_first = false;
                }
                if ui.radio(self.slide_new_first, tr("Du plus récent au plus ancien")).clicked() {
                    self.slide_new_first = true;
                }
                if ui.add(egui::Button::new(ic(icons::FULLSCREEN, tr("Plein écran"))).shortcut_text("F11")).clicked() {
                    self.fullscreen = !self.fullscreen;
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.fullscreen));
                    ui.close_menu();
                }
            });
        });
        }
        let dims = handle.as_ref().map(|t| format!("{}×{}", t.size()[0], t.size()[1])).unwrap_or_default();
        let when_fmt = format!("{} %H:%M:%S", dfmt(true));
        let when = chrono::DateTime::<chrono::Local>::from(modified).format(&when_fmt);
        if !self.fullscreen {
            ui.weak(format!("{}  ·  {dims}  ·  {}  ·  {when}  ·  {}/{}", file_name(&path), human(size), pos + 1, vis.len()));
        }

        if pin {
            let s = path.display().to_string();
            let _ = spawn_self(&["--pin", s.as_str()]);
        }
        if cfile {
            copy_files(&[path.clone()]);
            self.status = ic(icons::COPY, tr("Fichier copié"));
        }
        if annotate {
            self.editor = Editor::open(ctx, &path);
            return;
        }
        if edit {
            self.edit(&path);
        }
        if cpath {
            copy_text(&path.display().to_string());
            self.status = ic(icons::LINK, tr("Chemin copié"));
        }
        if ren {
            let stem = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
            self.rename = Some((path.clone(), stem));
        }
        if self.rename.as_ref().map(|r| r.0 != path).unwrap_or(false) {
            self.rename = None;
        }
        let (mut do_rename, mut cancel) = (None::<String>, false);
        if let Some((_, buf)) = &mut self.rename {
            ui.horizontal(|ui| {
                ui.label(tr("Nouveau nom :"));
                ui.add(egui::TextEdit::singleline(buf).desired_width(320.0));
                if ui.button(tr("Valider")).clicked() {
                    do_rename = Some(buf.clone());
                }
                if ui.button(tr("Annuler")).clicked() {
                    cancel = true;
                }
            });
        }
        if cancel {
            self.rename = None;
        }
        if let Some(n) = do_rename {
            self.rename = None;
            self.rename_file(&path, &n);
            return;
        }

        let k = ctx.input(|i| {
            (
                i.key_pressed(Key::ArrowLeft),
                i.key_pressed(Key::ArrowRight),
                i.key_pressed(Key::Escape),
                i.key_pressed(Key::Delete),
                i.key_pressed(Key::F),
                i.key_pressed(Key::F11),
                i.key_pressed(Key::Space),
            )
        });
        // pas de raccourcis pendant qu'on tape un nom
        let k = if ctx.wants_keyboard_input() { (false, false, false, false, false, false, false) } else { k };
        if k.0 {
            step = -1;
        }
        if k.1 {
            step = 1;
        }
        if k.2 {
            if self.fullscreen {
                self.fullscreen = false;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
            } else {
                self.view = None;
            }
        }
        if k.3 {
            del = true;
        }
        if k.4 {
            tfav = true;
        }
        if k.6 {
            self.slideshow = !self.slideshow;
            self.slide_at = Instant::now();
            if self.slideshow && !self.fullscreen {
                self.status = ic(icons::PLAY, &trf("Diaporama : {0}", &[&self.slide_label()]));
            }
        }
        if self.slideshow {
            ctx.request_repaint_after(Duration::from_millis(500));
            if self.slide_at.elapsed() >= Duration::from_secs(3) {
                self.slide_at = Instant::now();
                self.view = Some(self.slide_next(vis, v));
                self.view_dir = 1.0;
                return;
            }
        }
        if tfav {
            self.toggle_fav(path.clone());
        }
        if del {
            self.trash(vec![path]);
            self.view = None;
            return;
        }
        let n = pos as i32 + step;
        if step != 0 && n >= 0 && (n as usize) < vis.len() {
            self.view = Some(vis[n as usize]);
            self.view_dir = step as f32;
            return;
        }
        if self.view.is_none() {
            return;
        }

        if let Some(t) = handle {
            // zoom : molette vers le curseur, glisser pour déplacer, double-clic, + − 0 1 ; barre flottante en bas
            let s = t.size_vec2();
            let (area, resp) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
            let keys = !ctx.wants_keyboard_input() && self.rename.is_none();
            let r = self.zoom.update(ui, &resp, area, s, crate::zoom::Pan::Primary, keys, true);
            self.view_rect = Some(r);
            // ouverture depuis la grille : l'image vole depuis sa miniature (dessinée par-dessus, voir `Fly`)
            if let Some(from) = self.fly_from.take() {
                self.fly = Some(Fly { tex: t.clone(), from, to: r, t0: Instant::now(), closing: false, fade: false, idx: v });
                self.view_at = Instant::now() - Duration::from_secs(2);
            }
            let flying = match self.fly.as_mut() {
                Some(f) if !f.closing => {
                    f.to = r;
                    true
                }
                _ => false,
            };
            // ouverture sans miniature : l'image grandit en fondu ; précédente / suivante : elle glisse depuis le côté
            let k = ease_out((self.view_at.elapsed().as_secs_f32() / anim::ENTER).min(1.0));
            if k < 1.0 {
                ctx.request_repaint();
            }
            let dir = self.view_dir;
            let grow = if dir == 0.0 { 0.9 + 0.1 * k } else { 1.0 };
            let shown = Rect::from_center_size(r.center() + vec2(dir * 70.0 * (1.0 - k), 0.0), r.size() * grow);
            if !flying {
                ui.painter_at(area).image(t.id(), shown, uv(), Color32::from_white_alpha((k * 255.0) as u8));
            }
            self.zoom.controls(ui, area, tr("Molette : zoom · glisser : déplacer · double-clic : ajuster / taille réelle"));
        }
    }
}

/// Tuile de statistique : icône et nom en haut, chiffre en grand en bas à droite. Elle monte en fondu.
#[allow(clippy::too_many_arguments)]
fn stat_tile(ui: &mut egui::Ui, w: f32, h: f32, icon: &str, tint: Color32, solid: bool, label: &str, value: &str, unit: &str, sub: &str, since: f32, n: usize) {
    let (r, _) = ui.allocate_exact_size(vec2(w, h), Sense::hover());
    let k = ease_out(((since - 0.05 * n as f32) / 0.5).clamp(0.0, 1.0));
    if k < 1.0 {
        ui.ctx().request_repaint();
    }
    let v = ui.visuals().clone();
    let mut p = ui.painter().clone();
    p.multiply_opacity(k);
    let r = r.translate(vec2(0.0, 10.0 * (1.0 - k)));
    p.rect_filled(r, 14.0, v.extreme_bg_color);
    tile_content(&p, r, icon, tint, solid, label, value, unit, sub, &v);
}

/// Contenu d'une tuile (ou d'une moitié de tuile double).
#[allow(clippy::too_many_arguments)]
fn tile_content(p: &egui::Painter, r: Rect, icon: &str, tint: Color32, solid: bool, label: &str, value: &str, unit: &str, sub: &str, v: &egui::Visuals) {
    let inner = r.shrink2(vec2(18.0, 16.0));
    let tile = Rect::from_min_size(inner.min, vec2(30.0, 30.0));
    let dark = v.dark_mode;
    if solid {
        p.rect_filled(tile, 9.0, tint);
        p.text(tile.center(), Align2::CENTER_CENTER, icon, FontId::proportional(15.0), on_accent(tint));
    } else {
        p.rect_filled(tile, 9.0, Color32::from_rgba_unmultiplied(tint.r(), tint.g(), tint.b(), if dark { 42 } else { 52 }));
        let fg = if dark { tint.lerp_to_gamma(Color32::WHITE, 0.3) } else { tint.lerp_to_gamma(Color32::BLACK, 0.3) };
        p.text(tile.center(), Align2::CENTER_CENTER, icon, FontId::proportional(15.0), fg);
    }
    let fit = |text: &str, font: FontId, color: Color32, max_w: f32| fit_galley(p, text, font, color, max_w);
    let lab = fit(label, FontId::new(13.5, egui::FontFamily::Name("bold".into())), v.text_color(), inner.max.x - tile.max.x - 10.0);
    p.galley(pos2(tile.max.x + 10.0, tile.center().y - lab.size().y / 2.0), lab, v.text_color());
    // chiffre en bas à droite, unité plus petite et plus discrète
    let big = FontId::new(30.0, egui::FontFamily::Name("bold".into()));
    let mut x = inner.max.x;
    if !unit.is_empty() {
        let u = p.text(pos2(x, inner.max.y), Align2::RIGHT_BOTTOM, unit, FontId::new(16.0, egui::FontFamily::Name("bold".into())), v.weak_text_color());
        x = u.min.x - 5.0;
    }
    let vr = p.text(pos2(x, inner.max.y + 3.0), Align2::RIGHT_BOTTOM, value, big, v.text_color());
    if !sub.is_empty() {
        // nom du fichier, entre le titre et le chiffre
        let g = fit(sub, FontId::proportional(12.0), v.weak_text_color(), inner.width());
        let y = (tile.max.y + vr.min.y) / 2.0 - g.size().y / 2.0;
        p.galley(pos2(inner.min.x, y), g, v.weak_text_color());
    }
}

/// Texte sur une ligne, coupé avec « … » s'il dépasse `max_w`.
fn fit_galley(p: &egui::Painter, text: &str, font: FontId, color: Color32, max_w: f32) -> std::sync::Arc<egui::Galley> {
    let mut job = egui::text::LayoutJob::simple_singleline(text.to_owned(), font, color);
    job.wrap = egui::text::TextWrapping { max_width: max_w.max(10.0), max_rows: 1, break_anywhere: true, overflow_character: Some('…') };
    p.layout_job(job)
}

fn bold(size: f32) -> FontId {
    FontId::new(size, egui::FontFamily::Name("bold".into()))
}

/// Hauteur du titre d'un panneau (avec l'espace qui le suit).
const PANEL_TOP: f32 = 34.0;
/// Hauteur d'une ligne de la liste des dossiers (avec l'espace qui la suit).
const FOLDER_STEP: f32 = 48.0;

/// Panneau des statistiques : fond en retrait, titre (icône + nom) et, à droite, une info discrète.
/// Il arrive en fondu (`n` : son rang, pour le décalage). Renvoie la zone libre sous le titre et un
/// pinceau à l'opacité du fondu.
#[allow(clippy::too_many_arguments)]
fn stat_panel(ui: &mut egui::Ui, size: Vec2, since: f32, n: usize, icon: &str, title: &str, right: &str) -> (Rect, egui::Painter) {
    let (r, _) = ui.allocate_exact_size(size, Sense::hover());
    let k = ease_out(((since - 0.1 - 0.06 * n as f32) / 0.5).clamp(0.0, 1.0));
    if k < 1.0 {
        ui.ctx().request_repaint();
    }
    let v = ui.visuals();
    let mut p = ui.painter().clone();
    p.multiply_opacity(k);
    let r = r.translate(vec2(0.0, 10.0 * (1.0 - k)));
    p.rect_filled(r, 14.0, v.extreme_bg_color);
    let inner = r.shrink(20.0);
    let y = inner.min.y + 11.0;
    let ig = p.text(pos2(inner.min.x, y), Align2::LEFT_CENTER, icon, FontId::proportional(15.0), v.weak_text_color());
    let tg = fit_galley(&p, title, bold(15.0), v.text_color(), inner.max.x - ig.max.x - 8.0);
    let title_end = ig.max.x + 8.0 + tg.size().x;
    p.galley(pos2(ig.max.x + 8.0, y - tg.size().y / 2.0), tg, v.text_color());
    let room = inner.max.x - title_end - 16.0;
    if !right.is_empty() && room > 40.0 {
        let g = fit_galley(&p, right, FontId::proportional(12.5), v.weak_text_color(), room);
        p.galley(pos2(inner.max.x - g.size().x, y - g.size().y / 2.0), g, v.weak_text_color());
    }
    (Rect::from_min_max(pos2(inner.min.x, inner.min.y + PANEL_TOP), inner.max), p)
}

/// « 7 derniers jours » : une barre par jour, aujourd'hui en couleur d'accent.
fn week_panel(ui: &mut egui::Ui, size: Vec2, since: f32, counts: &[(String, usize)], acc: Color32) {
    let sum: usize = counts.iter().map(|c| c.1).sum();
    let (area, p) = stat_panel(ui, size, since, 0, icons::CALENDAR, tr("7 derniers jours"), &n_captures(sum));
    let v = ui.visuals().clone();
    let max = counts.iter().map(|c| c.1).max().unwrap_or(1).max(1);
    let colw = area.width() / counts.len().max(1) as f32;
    let bw = (colw * 0.55).min(46.0);
    let (label_h, num_h) = (22.0, 22.0);
    let bar_max = area.height() - label_h - num_h;
    for (i, (label, c)) in counts.iter().enumerate() {
        // les barres poussent l'une après l'autre en arrivant sur la page
        let grow = ease_out(((since - 0.12 - i as f32 * 0.06) / 0.7).clamp(0.0, 1.0));
        if grow < 1.0 {
            ui.ctx().request_repaint();
        }
        let today_col = i + 1 == counts.len();
        let cx = area.min.x + colw * (i as f32 + 0.5);
        let h = if *c == 0 { 6.0 } else { (bar_max * *c as f32 / max as f32).max(10.0) } * grow;
        let bottom = area.max.y - label_h;
        let bar = Rect::from_min_max(pos2(cx - bw / 2.0, bottom - h), pos2(cx + bw / 2.0, bottom));
        let fill = if today_col {
            acc
        } else if *c > 0 {
            v.widgets.hovered.weak_bg_fill
        } else {
            v.faint_bg_color
        };
        p.rect_filled(bar, Rounding { nw: 9.0, ne: 9.0, sw: 4.0, se: 4.0 }, fill);
        let shown = (*c as f32 * grow).round() as usize;
        let num_col = if today_col { acc.lerp_to_gamma(v.text_color(), 0.25) } else if *c > 0 { v.text_color() } else { v.weak_text_color() };
        p.text(pos2(cx, bar.min.y - 6.0), Align2::CENTER_BOTTOM, shown.to_string(), bold(13.5), num_col);
        let lf = if today_col { bold(12.5) } else { FontId::proportional(12.5) };
        p.text(pos2(cx, area.max.y - 2.0), Align2::CENTER_BOTTOM, label, lf, if today_col { v.text_color() } else { v.weak_text_color() });
    }
}

/// « Dossiers / jeux » : autant de dossiers que la place le permet, les suivants regroupés en « N autres ».
fn folders_panel(ui: &mut egui::Ui, size: Vec2, since: f32, folders: &[(String, (usize, u64))], acc: Color32) {
    let (area, p) = stat_panel(ui, size, since, 1, icons::FOLDER, tr("Dossiers / jeux"), &n_folders(folders.len()));
    let v = ui.visuals().clone();
    let colors = [acc, col(244, 114, 182), col(56, 189, 248), col(251, 191, 36), col(52, 211, 153), col(251, 146, 60), col(167, 139, 250), col(45, 212, 191)];
    let fit_rows = (((area.height() + 8.0) / FOLDER_STEP).floor() as usize).max(1);
    let mut rows: Vec<(String, usize, u64, Color32)> = Vec::new();
    if folders.len() > fit_rows {
        let keep = fit_rows - 1;
        for (n, (name, (c, sz))) in folders[..keep].iter().enumerate() {
            rows.push((name.clone(), *c, *sz, colors[n % colors.len()]));
        }
        let rest = &folders[keep..];
        rows.push((
            trf("{0} autres dossiers", &[&rest.len()]),
            rest.iter().map(|f| f.1 .0).sum(),
            rest.iter().map(|f| f.1 .1).sum(),
            v.weak_text_color(),
        ));
    } else {
        for (n, (name, (c, sz))) in folders.iter().enumerate() {
            rows.push((name.clone(), *c, *sz, colors[n % colors.len()]));
        }
    }
    let top = rows.iter().map(|r| r.1).max().unwrap_or(1).max(1);
    for (n, (name, c, sz, color)) in rows.iter().enumerate() {
        let grow = ease_out(((since - 0.2 - n as f32 * 0.06) / 0.7).clamp(0.0, 1.0));
        if grow < 1.0 {
            ui.ctx().request_repaint();
        }
        let r = Rect::from_min_size(pos2(area.min.x, area.min.y + n as f32 * FOLDER_STEP), vec2(area.width(), FOLDER_STEP - 8.0));
        // nombre et poids calés à droite ; le nom prend le reste (coupé avec « … » s'il est long)
        let y = r.min.y + 10.0;
        let info = p.text(pos2(r.max.x, y), Align2::RIGHT_CENTER, format!("{} · {}", n_captures(*c), human(*sz)), FontId::proportional(12.5), v.weak_text_color());
        let g = fit_galley(&p, name, bold(14.0), v.text_color(), info.min.x - r.min.x - 12.0);
        p.galley(pos2(r.min.x, y - g.size().y / 2.0), g, v.text_color());
        let track = Rect::from_min_size(pos2(r.min.x, r.max.y - 8.0), vec2(r.width(), 8.0));
        p.rect_filled(track, 4.0, v.faint_bg_color);
        let fill_w = (track.width() * *c as f32 / top as f32).max(8.0) * grow;
        p.rect_filled(Rect::from_min_size(track.min, vec2(fill_w, track.height())), 4.0, *color);
    }
}

/// Jours d'affilée avec au moins une capture : la série en cours (jusqu'à aujourd'hui, ou hier si
/// rien n'a encore été capturé aujourd'hui) et le record.
fn streaks(per_day: &HashMap<chrono::NaiveDate, usize>, today: chrono::NaiveDate) -> (usize, usize) {
    let one = chrono::Duration::days(1);
    let mut d = if per_day.contains_key(&today) { today } else { today - one };
    let mut cur = 0;
    while per_day.contains_key(&d) {
        cur += 1;
        d -= one;
    }
    let mut days: Vec<&chrono::NaiveDate> = per_day.keys().collect();
    days.sort();
    let (mut best, mut run) = (0, 0);
    let mut prev: Option<chrono::NaiveDate> = None;
    for &d in days {
        run = if prev.map(|p| p + one == d).unwrap_or(false) { run + 1 } else { 1 };
        best = best.max(run);
        prev = Some(d);
    }
    (cur, best)
}

/// Calendrier d'activité : une case par jour (colonnes = semaines, lignes = lundi → dimanche),
/// plus la couleur est vive, plus il y a de captures. Le nombre de semaines suit la largeur.
fn heat_panel(ui: &mut egui::Ui, size: Vec2, since: f32, per_day: &HashMap<chrono::NaiveDate, usize>, today: chrono::NaiveDate, acc: Color32) {
    use chrono::Datelike;
    let (g, label_w, legend_h) = (4.0, 26.0, 30.0);
    let avail = vec2(size.x - 40.0 - label_w, size.y - 40.0 - PANEL_TOP - legend_h);
    let cell = ((avail.y + g) / 7.0 - g).clamp(10.0, 26.0).floor();
    let weeks = (((avail.x + g) / (cell + g)).floor() as usize).clamp(4, 53);
    let monday = today - chrono::Duration::days(today.weekday().num_days_from_monday() as i64);
    let start = monday - chrono::Duration::weeks(weeks as i64 - 1);
    let in_range = |d: &chrono::NaiveDate| *d >= start && *d <= today;
    let total: usize = per_day.iter().filter(|(d, _)| in_range(d)).map(|(_, c)| *c).sum();
    let max = per_day.iter().filter(|(d, _)| in_range(d)).map(|(_, c)| *c).max().unwrap_or(1).max(1);
    let title = trf("Activité des {0} dernières semaines", &[&weeks]);
    let (area, p) = stat_panel(ui, size, since, 2, icons::HISTORY, &title, &n_captures(total));
    let v = ui.visuals().clone();
    let inset = v.extreme_bg_color;
    let level = |c: usize| -> Color32 {
        if c == 0 {
            v.faint_bg_color
        } else {
            let t = [0.3, 0.52, 0.76, 1.0][((c as f32 / max as f32 * 4.0).ceil() as usize).clamp(1, 4) - 1];
            inset.lerp_to_gamma(acc, t)
        }
    };
    // grille et légende centrées dans la place disponible
    let grid = vec2(weeks as f32 * (cell + g) - g, 7.0 * (cell + g) - g);
    let block = vec2(label_w + grid.x, grid.y + legend_h);
    let o = area.min + ((area.size() - block) / 2.0).max(Vec2::ZERO);
    let gx = o.x + label_w;
    let grid_rect = Rect::from_min_size(pos2(gx, o.y), grid);
    let letters = weekday_letters();
    for (r, l) in letters.iter().enumerate() {
        if cell >= 15.0 || r % 2 == 0 {
            let y = o.y + r as f32 * (cell + g) + cell / 2.0;
            p.text(pos2(o.x, y), Align2::LEFT_CENTER, *l, FontId::proportional(11.5), v.weak_text_color());
        }
    }
    let hover = ui.interact(grid_rect, ui.id().with("heat"), Sense::hover());
    let hover_cell = hover.hover_pos().map(|q| (((q.x - gx) / (cell + g)).floor() as i64, ((q.y - o.y) / (cell + g)).floor() as i64));
    let mut tip: Option<String> = None;
    for c in 0..weeks {
        // les colonnes apparaissent de gauche à droite
        let k = ease_out(((since - 0.25 - c as f32 * 0.02) / 0.45).clamp(0.0, 1.0));
        if k < 1.0 {
            ui.ctx().request_repaint();
        }
        for r in 0..7 {
            let d = start + chrono::Duration::days((c * 7 + r) as i64);
            if d > today {
                break;
            }
            let n = per_day.get(&d).copied().unwrap_or(0);
            let cr = Rect::from_min_size(pos2(gx + c as f32 * (cell + g), o.y + r as f32 * (cell + g)), vec2(cell, cell));
            let fill = level(n);
            p.rect_filled(cr, (cell * 0.24).round(), fill.gamma_multiply(k));
            if d == today {
                p.rect_stroke(cr.shrink(0.5), (cell * 0.24).round(), Stroke::new(1.5_f32, v.text_color()));
            }
            if hover_cell == Some((c as i64, r as i64)) {
                p.rect_stroke(cr.expand(1.5), (cell * 0.24).round() + 1.5, Stroke::new(1.5_f32, v.text_color().gamma_multiply(0.7)));
                tip = Some(format!("{}\n{}", d.format(dfmt(true)), n_captures(n)));
            }
        }
    }
    if let Some(t) = tip {
        hover.on_hover_text_at_pointer(t);
    }
    // légende : séries à gauche, échelle « Moins … Plus » à droite
    let ly = grid_rect.max.y + legend_h / 2.0 + 3.0;
    let sw = 11.0;
    let mut x = grid_rect.max.x;
    let plus = p.text(pos2(x, ly), Align2::RIGHT_CENTER, tr("Plus"), FontId::proportional(11.5), v.weak_text_color());
    x = plus.min.x - 6.0;
    for t in [1.0, 0.76, 0.52, 0.3] {
        p.rect_filled(Rect::from_min_size(pos2(x - sw, ly - sw / 2.0), vec2(sw, sw)), 3.0, inset.lerp_to_gamma(acc, t));
        x -= sw + 3.0;
    }
    p.rect_filled(Rect::from_min_size(pos2(x - sw, ly - sw / 2.0), vec2(sw, sw)), 3.0, v.faint_bg_color);
    x -= sw + 6.0;
    let moins = p.text(pos2(x, ly), Align2::RIGHT_CENTER, tr("Moins"), FontId::proportional(11.5), v.weak_text_color());
    let (cur, best) = streaks(per_day, today);
    let s = trf("Série en cours : {0} · record : {1}", &[&n_days(cur), &n_days(best)]);
    let sg = fit_galley(&p, &s, FontId::proportional(12.0), v.weak_text_color(), moins.min.x - 16.0 - o.x);
    p.galley(pos2(o.x, ly - sg.size().y / 2.0), sg, v.weak_text_color());
}

/// « Heures de capture » : une barre par heure de la journée, l'heure la plus active en couleur d'accent.
fn hours_panel(ui: &mut egui::Ui, size: Vec2, since: f32, per_hour: &[usize; 24], acc: Color32) {
    let max = per_hour.iter().copied().max().unwrap_or(0);
    let peak = per_hour.iter().position(|&c| c == max).unwrap_or(0);
    let right = if max > 0 { trf("Pic : {0}", &[&hour_label(peak)]) } else { String::new() };
    let (area, p) = stat_panel(ui, size, since, 3, icons::CLOCK, tr("Heures de capture"), &right);
    let v = ui.visuals().clone();
    let colw = area.width() / 24.0;
    let bw = (colw * 0.62).clamp(3.0, 18.0);
    let label_h = 20.0;
    let bottom = area.max.y - label_h;
    let bar_max = area.height() - label_h - 4.0;
    let hover = ui.interact(area, ui.id().with("hours"), Sense::hover());
    let hov = hover.hover_pos().map(|q| ((q.x - area.min.x) / colw).floor() as i64);
    let mut tip: Option<String> = None;
    for (h, &c) in per_hour.iter().enumerate() {
        let grow = ease_out(((since - 0.3 - h as f32 * 0.015) / 0.6).clamp(0.0, 1.0));
        if grow < 1.0 {
            ui.ctx().request_repaint();
        }
        let cx = area.min.x + colw * (h as f32 + 0.5);
        let bh = if c == 0 { 4.0 } else { (bar_max * c as f32 / max.max(1) as f32).max(6.0) } * grow;
        let bar = Rect::from_min_max(pos2(cx - bw / 2.0, bottom - bh), pos2(cx + bw / 2.0, bottom));
        let top_r = (bw / 2.0).min(6.0);
        let fill = if max > 0 && h == peak {
            acc
        } else if c > 0 {
            v.widgets.hovered.weak_bg_fill
        } else {
            v.faint_bg_color
        };
        let hovered = hov == Some(h as i64);
        let fill = if hovered && h != peak { fill.lerp_to_gamma(v.text_color(), 0.15) } else { fill };
        p.rect_filled(bar, Rounding { nw: top_r, ne: top_r, sw: 2.0, se: 2.0 }, fill);
        if hovered {
            tip = Some(format!("{} – {}\n{}", hour_label(h), hour_label((h + 1) % 24), n_captures(c)));
        }
        if h % 6 == 0 {
            p.text(pos2(cx, area.max.y - 2.0), Align2::CENTER_BOTTOM, hour_label(h), FontId::proportional(12.0), v.weak_text_color());
        }
    }
    if let Some(t) = tip {
        hover.on_hover_text_at_pointer(t);
    }
}
