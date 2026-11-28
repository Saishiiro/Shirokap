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
        let mut swap = false;
        contained(ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
            split_row(
                ui,
                "compare_bar",
                |ui| {
                    if ghost(ui, icons::BACK, tr("← Retour").trim_start_matches("← ")).on_hover_text(tr("Retour à la galerie (Échap)")).clicked() {
                        close = true;
                    }
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 1.0;
                        ui.label(bold_text(tr("Comparaison"), 15.0));
                        ui.label(egui::RichText::new(tr("Glisse sur l'image pour déplacer la séparation")).size(12.5).color(ui.visuals().weak_text_color()));
                    });
                },
                |ui| {
                    if pill(ui, icons::COMPARE, tr("Inverser A et B")).clicked() {
                        swap = true;
                    }
                },
            );
        });
        if let Some(c) = &mut self.compare {
            if swap {
                std::mem::swap(&mut c.a, &mut c.b);
                std::mem::swap(&mut c.na, &mut c.nb);
            }
            let (area, resp) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
            let v = ui.visuals().clone();
            ui.painter().rect_filled(area, 14.0, v.extreme_bg_color);
            let rect = area.shrink(12.0);
            let s = c.a.size_vec2();
            let k = (rect.width() / s.x).min(rect.height() / s.y);
            let ir = Rect::from_center_size(rect.center(), s * k);
            if resp.hovered() || resp.dragged() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
            }
            if let Some(pos) = resp.interact_pointer_pos() {
                if resp.dragged() || resp.clicked() {
                    c.split = ((pos.x - ir.min.x) / ir.width()).clamp(0.0, 1.0);
                }
            }
            let p = ui.painter_at(area);
            p.image(c.a.id(), ir, uv(), Color32::WHITE);
            let x = ir.min.x + ir.width() * c.split;
            let right = Rect::from_min_max(pos2(x, ir.min.y), ir.max);
            p.image(c.b.id(), right, Rect::from_min_max(pos2(c.split, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
            // séparation : un trait blanc ombré et une poignée ronde au milieu
            p.line_segment([pos2(x, ir.min.y), pos2(x, ir.max.y)], Stroke::new(4.0_f32, Color32::from_black_alpha(70)));
            p.line_segment([pos2(x, ir.min.y), pos2(x, ir.max.y)], Stroke::new(2.0_f32, Color32::WHITE));
            let knob = Rect::from_center_size(pos2(x, ir.center().y), vec2(36.0, 36.0));
            let shadow = egui::epaint::Shadow { offset: vec2(0.0, 3.0), blur: 12.0, spread: 0.0, color: Color32::from_black_alpha(100) };
            p.add(shadow.as_shape(knob, 18.0));
            p.rect_filled(knob, 18.0, Color32::WHITE);
            let dark = Color32::from_gray(40);
            let st = Stroke::new(1.8_f32, dark);
            let c0 = knob.center();
            p.line_segment([c0 + vec2(-4.0, -5.0), c0 + vec2(-9.0, 0.0)], st);
            p.line_segment([c0 + vec2(-9.0, 0.0), c0 + vec2(-4.0, 5.0)], st);
            p.line_segment([c0 + vec2(4.0, -5.0), c0 + vec2(9.0, 0.0)], st);
            p.line_segment([c0 + vec2(9.0, 0.0), c0 + vec2(4.0, 5.0)], st);
            // étiquettes A et B dans les coins de l'image
            let max_w = (ir.width() / 2.0 - 24.0).max(60.0);
            corner_tag(&p, pos2(ir.min.x + 10.0, ir.min.y + 10.0), false, "A", &c.na, max_w);
            corner_tag(&p, pos2(ir.max.x - 10.0, ir.min.y + 10.0), true, "B", &c.nb, max_w);
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
        let (mut annotate, mut pin, mut cfile, mut back, mut copy) = (false, false, false, false, false);
        let (mut open_ext, mut show_dir, mut toggle_show, mut toggle_full) = (false, false, false, false);
        let mut new_first = self.slide_new_first;
        let slideshow = self.slideshow;

        let danger = col(239, 68, 68);
        let gold = col(251, 191, 36);
        let dims = handle.as_ref().map(|t| format!("{}×{}", t.size()[0], t.size()[1])).unwrap_or_default();
        let when_fmt = format!("{} %H:%M", dfmt(true));
        let when = chrono::DateTime::<chrono::Local>::from(modified).format(&when_fmt).to_string();
        let mut bar_gap: Option<Rect> = None;
        if !self.fullscreen {
            // barre du haut : retour et nom de la capture à gauche ; actions calées à droite
            let name = file_name(&path);
            let short: String = if name.chars().count() > 46 { name.chars().take(44).collect::<String>() + "…" } else { name.clone() };
            let details = format!("{dims}  ·  {}  ·  {when}  ·  {}/{}", human(size), pos + 1, vis.len());
            let groups = contained(ui, |ui| {
                ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
                split_row(
                    ui,
                    "viewer_bar",
                    |ui| {
                        if ghost(ui, icons::BACK, tr("← Retour").trim_start_matches("← ")).on_hover_text(tr("Retour à la galerie (Échap)")).clicked() {
                            back = true;
                        }
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 1.0;
                            ui.label(bold_text(&short, 15.0)).on_hover_text(&name);
                            ui.label(egui::RichText::new(&details).size(12.5).color(ui.visuals().weak_text_color()));
                        });
                    },
                    |ui| {
                        if pill(ui, icons::COPY, tr("Copier")).on_hover_text(tr("Copier l'image dans le presse-papiers")).clicked() {
                            copy = true;
                        }
                        if accent_pill(ui, icons::ANNOTATE, tr("Annoter")).on_hover_text(tr("Ouvrir l'éditeur d'annotations")).clicked() {
                            annotate = true;
                        }
                        let (fav_tip, fav_col) = if fav { (tr("Retirer des favoris (F)"), gold) } else { (tr("Ajouter aux favoris (F)"), ui.visuals().text_color()) };
                        if tinted_pill(ui, icons::STAR, tr("Favori"), fav_col).on_hover_text(fav_tip).clicked() {
                            tfav = true;
                        }
                        if tinted_pill(ui, icons::TRASH, tr("Corbeille"), danger).on_hover_text(tr("Mettre à la corbeille (Suppr)")).clicked() {
                            del = true;
                        }
                        dropdown(ui, &ic(icons::MORE, tr("Plus…")), false, |ui| {
                            let item = |ui: &mut egui::Ui, icon: &str, label: &str, key: &str, flag: &mut bool| {
                                if menu_item(ui, icon, label, false, key).clicked() {
                                    *flag = true;
                                    close_dropdown(ui);
                                }
                            };
                            item(ui, icons::RENAME, tr("Renommer"), "", &mut ren);
                            item(ui, icons::PIN, tr("Épingler à l'écran"), "", &mut pin);
                            item(ui, icons::EDIT_EXT, tr("Modifier dans l'éditeur externe"), "", &mut edit);
                            ui.separator();
                            item(ui, icons::EXTERNAL, tr("Ouvrir avec l'application par défaut"), "", &mut open_ext);
                            item(ui, icons::FOLDER_OPEN, tr("Afficher dans le dossier"), "", &mut show_dir);
                            item(ui, icons::FOLDER, tr("Copier le fichier"), "", &mut cfile);
                            item(ui, icons::LINK, tr("Copier le chemin"), "", &mut cpath);
                            ui.separator();
                            let (si, sl) = if slideshow { (icons::PAUSE, tr("Arrêter le diaporama")) } else { (icons::PLAY, tr("Diaporama")) };
                            item(ui, si, sl, tr("Espace"), &mut toggle_show);
                            // ordre du diaporama : la liste reste ouverte pour voir le choix
                            menu_label(ui, tr("Ordre du diaporama"));
                            if menu_item(ui, icons::OLDEST, tr("Du plus ancien au plus récent"), !new_first, "").clicked() {
                                new_first = false;
                            }
                            if menu_item(ui, icons::NEWEST, tr("Du plus récent au plus ancien"), new_first, "").clicked() {
                                new_first = true;
                            }
                            ui.separator();
                            item(ui, icons::FULLSCREEN, tr("Plein écran"), "F11", &mut toggle_full);
                        });
                    },
                )
            });
            // place libre entre le nom et les boutons : la pilule du diaporama s'y loge
            bar_gap = groups.map(|(l, r)| Rect::from_min_max(pos2(l.max.x + 16.0, l.min.y), pos2(r.min.x - 16.0, r.max.y)));
        }
        self.slide_new_first = new_first;
        if back {
            self.view = None;
            return;
        }
        if copy {
            if let Ok(i) = image::open(&path) {
                copy_to_clipboard(&i.to_rgba8());
                self.status = ic(icons::COPY, tr("Image copiée dans le presse-papiers"));
            }
        }
        if open_ext {
            let _ = Command::new("explorer").arg(&path).spawn();
        }
        if show_dir {
            let _ = Command::new("explorer").arg("/select,").arg(&path).spawn();
        }
        if toggle_full {
            self.toggle_fullscreen(ctx);
        }
        if pin {
            let s = path.display().to_string();
            let _ = spawn_self(&["--pin", s.as_str()]);
        }
        if cfile {
            copy_files(std::slice::from_ref(&path));
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
            self.rename_focus = true;
        }
        if self.rename.as_ref().map(|r| r.0 != path).unwrap_or(false) {
            self.rename = None;
        }
        // renommage : un champ sous la barre (Entrée pour valider)
        let (mut do_rename, mut cancel) = (None::<String>, false);
        let focus = std::mem::take(&mut self.rename_focus);
        if let Some((_, buf)) = self.rename.as_mut() {
            contained(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    ui.label(egui::RichText::new(tr("Nouveau nom :")).color(ui.visuals().weak_text_color()));
                    let r = field(ui, buf, "", 320.0);
                    if focus {
                        r.request_focus();
                    }
                    let enter = r.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
                    if accent_pill(ui, icons::RENAME, tr("Renommer")).clicked() || enter {
                        do_rename = Some(buf.clone());
                    }
                    if ghost(ui, "", tr("Annuler")).clicked() {
                        cancel = true;
                    }
                });
            });
            ui.add_space(4.0);
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
        // (ni pendant la confirmation de la corbeille : Échap et Entrée sont pour elle)
        let k = if ctx.wants_keyboard_input() || self.confirm_trash.is_some() { (false, false, false, false, false, false, false) } else { k };
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
        if k.6 || toggle_show {
            self.slideshow = !self.slideshow;
            self.slide_at = Instant::now();
            self.zoom.wake(ui); // la pilule du diaporama se montre un instant
            if self.slideshow && !self.fullscreen {
                self.status = ic(icons::PLAY, &trf("Diaporama : {0}", &[&self.slide_label()]));
            }
        }
        if self.slideshow {
            // la barre de progression a besoin d'images régulières ; cachée, on attend juste l'image suivante
            let left = Duration::from_secs(self.cfg.slide_secs.max(1) as u64).saturating_sub(self.slide_at.elapsed());
            let shown = !self.fullscreen || self.zoom.awake(ui);
            ctx.request_repaint_after(if shown { Duration::from_millis(100) } else { left });
            if self.slide_at.elapsed() >= Duration::from_secs(self.cfg.slide_secs.max(1) as u64) {
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
            self.confirm_trash = Some(vec![path.clone()]); // confirmation d'abord (voir `confirm_trash`)
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
            // l'image est posée dans un panneau en retrait (sauf en plein écran, sur fond noir)
            // zoom : molette vers le curseur, glisser pour déplacer, double-clic, + − 0 1 ; barre flottante en bas
            let s = t.size_vec2();
            let (area, resp) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
            if !self.fullscreen {
                ui.painter().rect_filled(area, 14.0, ui.visuals().extreme_bg_color);
            }
            let inner = if self.fullscreen { area } else { area.shrink(12.0) };
            let keys = !ctx.wants_keyboard_input() && self.rename.is_none() && self.confirm_trash.is_none();
            let r = self.zoom.update(ui, &resp, inner, s, crate::zoom::Pan::Primary, keys, true);
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

            // précédente / suivante : boutons ronds sur les côtés, visibles quand la souris est sur l'image
            let near = ui.rect_contains_pointer(area);
            let nk = ctx.animate_bool_with_time(egui::Id::new("viewer_nav"), near, 0.2);
            let mut go: Option<i32> = None;
            if nk > 0.01 {
                for (d, icon, ok, x, tip) in [
                    (-1, icons::PREV, pos > 0, area.min.x + 36.0, tr("Précédente (←)")),
                    (1, icons::NEXT, pos + 1 < vis.len(), area.max.x - 36.0, tr("Suivante (→)")),
                ] {
                    if ok && nav_button(ui, egui::Id::new(("viewer_nav", d)), pos2(x, area.center().y), icon, nk).on_hover_text(tip).clicked() {
                        go = Some(d);
                    }
                }
            }
            // diaporama en cours : petite pilule avec le temps restant avant l'image suivante, au centre de la
            // barre du haut ; en plein écran (ou si la barre est trop étroite), elle flotte en haut de l'image
            // et, comme la barre de zoom, ne se montre que quand la souris bouge
            if self.slideshow {
                let frac = (self.slide_at.elapsed().as_secs_f32() / self.cfg.slide_secs.max(1) as f32).min(1.0);
                let size = slide_pill_size(ui);
                let clicked = match bar_gap.filter(|g| g.width() >= size.x) {
                    Some(g) => {
                        let k = ctx.animate_bool_with_time(egui::Id::new("slide_pill_bar"), true, 0.25);
                        slide_pill(ui, g.center(), frac, k, ui.clip_rect()).clicked()
                    }
                    None => {
                        let home = Rect::from_center_size(pos2(area.center().x, area.min.y + 18.0 + size.y / 2.0), size);
                        if ui.rect_contains_pointer(home.expand(12.0)) {
                            self.zoom.wake(ui); // tant que la souris est dessus, elle reste là
                        }
                        let k = ctx.animate_bool_with_time_and_easing(egui::Id::new("slide_pill_k"), self.zoom.awake(ui), 0.3, egui::emath::easing::cubic_out);
                        // elle remonte en s'effaçant (la barre de zoom, elle, descend)
                        k > 0.02 && slide_pill(ui, home.center() - vec2(0.0, 12.0 * (1.0 - k)), frac, k, area).clicked()
                    }
                };
                if clicked {
                    self.slideshow = false;
                }
            } else {
                ctx.animate_bool_with_time(egui::Id::new("slide_pill_bar"), false, 0.0);
            }
            self.zoom.controls(ui, area, tr("Molette : zoom · glisser : déplacer · double-clic : ajuster / taille réelle"));
            if let Some(d) = go {
                let n = pos as i32 + d;
                if n >= 0 && (n as usize) < vis.len() {
                    self.view = Some(vis[n as usize]);
                    self.view_dir = d as f32;
                    ctx.request_repaint();
                }
            }
        }
    }
}

/// Étiquette « A · nom » posée dans un coin de l'image du comparateur (`right` : calée à droite de `at`).
fn corner_tag(p: &egui::Painter, at: Pos2, right: bool, letter: &str, name: &str, max_w: f32) {
    let name_g = fit_galley(p, name, FontId::proportional(12.5), Color32::WHITE, max_w - 40.0);
    let size = vec2(name_g.size().x + 44.0, 28.0);
    let r = if right { Rect::from_min_size(pos2(at.x - size.x, at.y), size) } else { Rect::from_min_size(at, size) };
    p.rect_filled(r, 14.0, Color32::from_black_alpha(170));
    let badge = Rect::from_center_size(pos2(r.min.x + 15.0, r.center().y), vec2(20.0, 20.0));
    p.rect_filled(badge, 10.0, Color32::WHITE);
    p.text(badge.center(), Align2::CENTER_CENTER, letter, bold(12.0), Color32::from_gray(30));
    p.galley(pos2(badge.max.x + 8.0, r.center().y - name_g.size().y / 2.0), name_g, Color32::WHITE);
}

/// Bouton rond flottant (précédente / suivante de la visionneuse), qui apparaît en fondu (`k`).
fn nav_button(ui: &mut egui::Ui, id: egui::Id, center: Pos2, icon: &str, k: f32) -> egui::Response {
    let r = Rect::from_center_size(center, vec2(46.0, 46.0));
    let resp = ui.interact(r, id, Sense::click());
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let hk = ui.ctx().animate_bool_with_time(id.with("hov"), resp.hovered(), anim::HOVER);
    let v = ui.visuals();
    let mut p = ui.painter().clone();
    p.multiply_opacity(k);
    let r = r.expand(2.0 * hk);
    let shadow = egui::epaint::Shadow { offset: vec2(0.0, 4.0), blur: 16.0, spread: 0.0, color: Color32::from_black_alpha(90) };
    p.add(shadow.as_shape(r, r.height() / 2.0));
    p.rect_filled(r, r.height() / 2.0, v.window_fill.lerp_to_gamma(v.widgets.hovered.weak_bg_fill, hk).gamma_multiply(0.96));
    p.text(r.center(), Align2::CENTER_CENTER, icon, FontId::proportional(20.0), v.text_color());
    resp
}

fn slide_pill_label() -> String {
    ic(icons::PAUSE, tr("Diaporama · clic ou Espace : arrêter"))
}

fn slide_pill_size(ui: &egui::Ui) -> Vec2 {
    let g = ui.painter().layout_no_wrap(slide_pill_label(), FontId::proportional(13.0), Color32::PLACEHOLDER);
    vec2(g.size().x + 32.0, 34.0)
}

/// Pilule « Diaporama » centrée sur `center` : barre de progression jusqu'à l'image suivante ; un clic
/// l'arrête. `k` : apparition (opacité), `clip` : zone où elle peut être dessinée.
fn slide_pill(ui: &mut egui::Ui, center: Pos2, frac: f32, k: f32, clip: Rect) -> egui::Response {
    let v = ui.visuals().clone();
    let g = ui.painter().layout_no_wrap(slide_pill_label(), FontId::proportional(13.0), v.text_color());
    let r = Rect::from_center_size(center, slide_pill_size(ui));
    let resp = ui.interact(r, egui::Id::new("slide_pill"), if k > 0.5 { Sense::click() } else { Sense::hover() });
    if resp.hovered() && k > 0.5 {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let mut p = ui.painter().with_clip_rect(clip.intersect(ui.clip_rect()));
    p.multiply_opacity(k);
    let shadow = egui::epaint::Shadow { offset: vec2(0.0, 4.0), blur: 16.0, spread: 0.0, color: Color32::from_black_alpha(90) };
    p.add(shadow.as_shape(r, 17.0));
    p.rect_filled(r, 17.0, v.window_fill.gamma_multiply(0.96));
    p.galley(pos2(r.min.x + 16.0, r.center().y - g.size().y / 2.0 - 2.0), g, v.text_color());
    let track = Rect::from_min_size(pos2(r.min.x + 16.0, r.max.y - 7.0), vec2(r.width() - 32.0, 3.0));
    p.rect_filled(track, 1.5, v.widgets.inactive.weak_bg_fill);
    p.rect_filled(Rect::from_min_size(track.min, vec2(track.width() * frac, 3.0)), 1.5, accent_of(&v));
    resp
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
