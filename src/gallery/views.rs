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
        if self.items.is_empty() {
            let text = trf("Appuie sur {0} pour capturer l'écran, ou {1} pour une zone : tes captures apparaîtront ici.", &[&self.cfg.hotkey, &self.cfg.hotkey_region]);
            if empty_view(ui, self.tab_at, self.shown_acc, icons::STATS, tr("Pas encore de statistiques"), &text, &ic(icons::GALLERY, tr("Aller à la galerie"))) {
                self.tab = Tab::Gallery;
            }
            return;
        }
        let acc = self.shown_acc;
        let since = self.tab_at.elapsed().as_secs_f32();
        egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
            ui.set_max_width(ui.available_width() - 12.0);
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

            // --- 7 derniers jours (barres verticales) et répartition par dossier, côte à côte ---
            let counts: Vec<(String, usize)> = (0..7)
                .rev()
                .map(|d| {
                    let day = today - chrono::Duration::days(d);
                    let label = if d == 0 { tr("Aujourd'hui").to_string() } else { day.format(dfmt(false)).to_string() };
                    (label, self.items.iter().filter(|i| day_of(i.modified) == day).count())
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
            folders.sort_by(|a, b| b.1 .0.cmp(&a.1 .0));
            folders.truncate(8);
            let w = ui.available_width();
            let side = w > 760.0;
            let (cw, fw) = if side { let c = ((w - gap) * 0.6).floor(); (c, w - gap - c) } else { (w, w) };
            let panel_h = 300.0_f32.max(76.0 + folders.len() as f32 * 44.0);
            let chart = |ui: &mut egui::Ui| {
                egui::Frame::none().fill(inset).rounding(14.0).inner_margin(egui::Margin::same(20.0)).show(ui, |ui| {
                    ui.set_width(cw - 40.0);
                    ui.set_height(panel_h - 40.0);
                    // le panneau peut être placé dans une ligne : son contenu, lui, s'empile de haut en bas
                    ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(icons::CALENDAR).color(v.weak_text_color()));
                        ui.label(bold_text(tr("7 derniers jours"), 15.0));
                    });
                    ui.add_space(8.0);
                    let (area, _) = ui.allocate_exact_size(ui.available_size(), Sense::hover());
                    let max = counts.iter().map(|c| c.1).max().unwrap_or(1).max(1);
                    let n = counts.len() as f32;
                    let colw = area.width() / n;
                    let bw = (colw * 0.55).min(46.0);
                    let label_h = 22.0;
                    let num_h = 22.0;
                    let bar_max = area.height() - label_h - num_h;
                    let p = ui.painter();
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
                        p.text(pos2(cx, bar.min.y - 6.0), Align2::CENTER_BOTTOM, shown.to_string(), FontId::new(13.5, egui::FontFamily::Name("bold".into())), num_col);
                        let lf = if today_col { FontId::new(12.5, egui::FontFamily::Name("bold".into())) } else { FontId::proportional(12.5) };
                        p.text(pos2(cx, area.max.y - 2.0), Align2::CENTER_BOTTOM, label, lf, if today_col { v.text_color() } else { v.weak_text_color() });
                    }
                    });
                });
            };
            let palette_cols = [acc, col(244, 114, 182), col(56, 189, 248), col(251, 191, 36), col(52, 211, 153), col(251, 146, 60), col(167, 139, 250), col(148, 163, 184)];
            let folders_ui = |ui: &mut egui::Ui| {
                egui::Frame::none().fill(inset).rounding(14.0).inner_margin(egui::Margin::same(20.0)).show(ui, |ui| {
                    ui.set_width(fw - 40.0);
                    ui.set_min_height(panel_h - 40.0);
                    ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(icons::FOLDER).color(v.weak_text_color()));
                        ui.label(bold_text(tr("Dossiers / jeux"), 15.0));
                    });
                    ui.add_space(6.0);
                    let top = folders.first().map(|f| f.1 .0).unwrap_or(1).max(1);
                    for (n, (name, (c, sz))) in folders.iter().enumerate() {
                        let grow = ease_out(((since - 0.2 - n as f32 * 0.06) / 0.7).clamp(0.0, 1.0));
                        // nombre et poids calés à droite ; le nom prend le reste (coupé avec « … » s'il est long)
                        ui.horizontal(|ui| {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(egui::RichText::new(format!("{} · {}", n_captures(*c), human(*sz))).size(12.5).color(v.weak_text_color()));
                                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                    ui.add(egui::Label::new(bold_text(name, 14.0)).truncate());
                                });
                            });
                        });
                        let (track, _) = ui.allocate_exact_size(vec2(ui.available_width(), 8.0), Sense::hover());
                        let p = ui.painter();
                        p.rect_filled(track, 4.0, v.faint_bg_color);
                        let fill_w = (track.width() * *c as f32 / top as f32).max(8.0) * grow;
                        p.rect_filled(Rect::from_min_size(track.min, vec2(fill_w, track.height())), 4.0, palette_cols[n % palette_cols.len()]);
                        if grow < 1.0 {
                            ui.ctx().request_repaint();
                        }
                        ui.add_space(6.0);
                    }
                    });
                });
            };
            if side {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = gap;
                    chart(ui);
                    folders_ui(ui);
                });
            } else {
                chart(ui);
                ui.add_space(gap);
                folders_ui(ui);
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
    let fit = |text: &str, font: FontId, color: Color32, max_w: f32| {
        let mut job = egui::text::LayoutJob::simple_singleline(text.to_owned(), font, color);
        job.wrap = egui::text::TextWrapping { max_width: max_w.max(10.0), max_rows: 1, break_anywhere: true, overflow_character: Some('…') };
        p.layout_job(job)
    };
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
