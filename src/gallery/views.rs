// Visionneuse, comparateur et statistiques.
use super::*;

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
        ui.label(bold_text(tr("Statistiques"), 24.0));
        ui.add_space(4.0);
        egui::ScrollArea::vertical().show(ui, |ui| {
            card(ui, tr("Vue d'ensemble"), |ui| {
                ui.label(format!("{} · {}", n_captures(self.items.len()), human(total)));
                ui.label(n_favs(self.favs.len()));
                if let Some(o) = self.items.last() {
                    ui.label(trf("Plus ancienne : {0}", &[&day_of(o.modified).format(dfmt(true))]));
                }
                if let Some(b) = self.items.iter().max_by_key(|i| i.size) {
                    ui.label(trf("Plus lourde : {0} ({1})", &[&file_name(&b.path), &human(b.size)]));
                }
            });
            card(ui, tr("7 derniers jours"), |ui| {
                let counts: Vec<(String, usize)> = (0..7)
                    .map(|d| {
                        let day = today - chrono::Duration::days(d);
                        let label = if d == 0 { tr("Aujourd'hui").to_string() } else { day.format(dfmt(false)).to_string() };
                        (label, self.items.iter().filter(|i| day_of(i.modified) == day).count())
                    })
                    .collect();
                let max = counts.iter().map(|c| c.1).max().unwrap_or(1).max(1);
                let since = self.tab_at.elapsed().as_secs_f32();
                for (row_i, (label, n)) in counts.into_iter().enumerate() {
                    let grow = ease_out(((since - 0.12 - row_i as f32 * 0.07) / 0.7).clamp(0.0, 1.0));
                    if grow < 1.0 {
                        ui.ctx().request_repaint();
                    }
                    let (row, _) = ui.allocate_exact_size(vec2(480.0, 22.0), Sense::hover());
                    let txt = ui.visuals().text_color();
                    let acc = ui.visuals().selection.bg_fill;
                    let track = ui.visuals().widgets.inactive.bg_fill;
                    let p = ui.painter();
                    p.text(row.left_center(), Align2::LEFT_CENTER, label, FontId::proportional(15.0), txt);
                    let bar = Rect::from_min_size(pos2(row.min.x + 120.0, row.center().y - 7.0), vec2(320.0, 14.0));
                    p.rect_filled(bar, 7.0, track);
                    if n > 0 && grow > 0.0 {
                        let w = (bar.width() * n as f32 / max as f32).max(14.0) * grow;
                        p.rect_filled(Rect::from_min_size(bar.min, vec2(w.max(1.0), bar.height())), 7.0, acc);
                    }
                    let shown_n = (n as f32 * grow).round() as usize;
                    p.text(pos2(bar.max.x + 12.0, row.center().y), Align2::LEFT_CENTER, shown_n.to_string(), FontId::proportional(15.0), txt);
                }
            });
            card(ui, tr("Dossiers / jeux"), |ui| {
                let mut by: HashMap<String, (usize, u64)> = HashMap::new();
                for i in &self.items {
                    let name = if i.folder.is_empty() { "(racine)".to_string() } else { i.folder.clone() };
                    let e = by.entry(name).or_default();
                    e.0 += 1;
                    e.1 += i.size;
                }
                let mut v: Vec<(String, (usize, u64))> = by.into_iter().collect();
                v.sort_by(|a, b| b.1 .0.cmp(&a.1 .0));
                for (name, (n, sz)) in v.into_iter().take(10) {
                    ui.label(format!("{name} : {} · {}", n_captures(n), human(sz)));
                }
            });
        });
    }

    pub(super) fn compare_ui(&mut self, ui: &mut egui::Ui) {
        let mut close = ui.input(|i| i.key_pressed(Key::Escape));
        ui.horizontal(|ui| {
            if ui.button(tr("← Retour")).clicked() {
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
            if ui.button(tr("← Retour")).clicked() {
                self.view = None;
            }
            if ui.add(egui::Button::new("‹").min_size(vec2(34.0, 30.0))).on_hover_text(tr("Précédente (←)")).clicked() {
                step = -1;
            }
            if ui.add(egui::Button::new("›").min_size(vec2(34.0, 30.0))).on_hover_text(tr("Suivante (→)")).clicked() {
                step = 1;
            }
            ui.separator();
            if ui.button(tr("Copier")).on_hover_text(tr("Copier l'image dans le presse-papiers")).clicked() {
                if let Ok(i) = image::open(&path) {
                    copy_to_clipboard(&i.to_rgba8());
                    self.status = tr("Image copiée dans le presse-papiers").into();
                }
            }
            let annot = egui::Button::new(egui::RichText::new(tr("Annoter")).color(on_accent(acc))).fill(acc);
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
            let trash = egui::Button::new(egui::RichText::new(tr("Corbeille")).color(danger));
            if ui.add(trash).on_hover_text(tr("Mettre à la corbeille (Suppr)")).clicked() {
                del = true;
            }
            dropdown(ui, tr("Plus…"), false, |ui| {
                if ui.button(tr("Renommer")).clicked() {
                    ren = true;
                    ui.close_menu();
                }
                if ui.button(tr("Épingler à l'écran")).clicked() {
                    pin = true;
                    ui.close_menu();
                }
                if ui.button(tr("Modifier dans l'éditeur externe")).clicked() {
                    edit = true;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button(tr("Ouvrir avec l'application par défaut")).clicked() {
                    let _ = Command::new("explorer").arg(&path).spawn();
                    ui.close_menu();
                }
                if ui.button(tr("Afficher dans le dossier")).clicked() {
                    let _ = Command::new("explorer").arg("/select,").arg(&path).spawn();
                    ui.close_menu();
                }
                if ui.button(tr("Copier le fichier")).clicked() {
                    cfile = true;
                    ui.close_menu();
                }
                if ui.button(tr("Copier le chemin")).clicked() {
                    cpath = true;
                    ui.close_menu();
                }
                ui.separator();
                let ss = egui::Button::new(if self.slideshow { tr("Arrêter le diaporama") } else { tr("Diaporama") }).shortcut_text(tr("Espace"));
                if ui.add(ss).clicked() {
                    self.slideshow = !self.slideshow;
                    self.slide_at = Instant::now();
                    if self.slideshow {
                        self.status = trf("Diaporama : {0}", &[&self.slide_label()]);
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
                if ui.add(egui::Button::new(tr("Plein écran")).shortcut_text("F11")).clicked() {
                    self.fullscreen = !self.fullscreen;
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.fullscreen));
                    ui.close_menu();
                }
            });
            ui.separator();
            if ui.button(tr("Ajuster")).on_hover_text(tr("Adapter à la fenêtre")).clicked() {
                self.zoom = None;
            }
            if ui.button("100 %").on_hover_text(tr("Taille réelle (Ctrl + molette pour zoomer)")).clicked() {
                self.zoom = Some(1.0);
            }
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
            self.status = tr("Fichier copié").into();
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
            self.status = tr("Chemin copié").into();
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
                self.status = trf("Diaporama : {0}", &[&self.slide_label()]);
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
            let s = t.size_vec2();
            let a = ui.available_size();
            let fit = (a.x / s.x).min(a.y / s.y).min(1.0);
            let zd = ctx.input(|i| i.zoom_delta()); // Ctrl + molette
            if zd != 1.0 {
                self.zoom = Some((self.zoom.unwrap_or(fit) * zd).clamp(0.05, 8.0));
            }
            match self.zoom {
                None => {
                    // ouverture : l'image grandit en fondu ; précédente / suivante : elle glisse depuis le côté
                    let k = ease_out((self.view_at.elapsed().as_secs_f32() / anim::ENTER).min(1.0));
                    if k < 1.0 {
                        ctx.request_repaint();
                    }
                    let dir = self.view_dir;
                    ui.vertical_centered(|ui| {
                        let (r, _) = ui.allocate_exact_size(s * fit, Sense::hover());
                        let scale = if dir == 0.0 { 0.9 + 0.1 * k } else { 1.0 };
                        let shown = Rect::from_center_size(r.center() + vec2(dir * 70.0 * (1.0 - k), 0.0), r.size() * scale);
                        ui.painter().image(t.id(), shown, uv(), Color32::from_white_alpha((k * 255.0) as u8));
                    });
                }
                Some(z) => {
                    egui::ScrollArea::both().show(ui, |ui| {
                        let (r, _) = ui.allocate_exact_size(s * z, Sense::hover());
                        ui.painter().image(t.id(), r, uv(), Color32::WHITE);
                    });
                }
            }
        }
    }
}
