// Grille de miniatures de la galerie.
use super::*;

impl App {
    // ----- galerie -----
    pub(super) fn gallery_ui(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        // glisser-déposer : on importe les images déposées sur la fenêtre
        let dropped: Vec<PathBuf> =
            ctx.input(|i| i.raw.dropped_files.iter().filter_map(|f| f.path.clone()).collect());
        if !dropped.is_empty() {
            let n = dropped.iter().filter(|p| import_file(&self.cfg, p)).count();
            self.items = list_items(&self.cfg.save_dir);
            self.status = ic(icons::IMAGE_PLUS, &trf("{0} image(s) importée(s)", &[&n]));
        }
        if let Some(ed) = &mut self.editor {
            match ed.ui(ui) {
                editor::Exit::None => {}
                editor::Exit::Close => self.editor = None,
                editor::Exit::Saved(p) => {
                    self.thumbs.remove(&p);
                    self.full = None;
                    self.items = list_items(&self.cfg.save_dir);
                    self.view = self.items.iter().position(|i| i.path == p);
                    self.view_dir = 0.0;
                    self.editor = None;
                    self.status = ic(icons::SUCCESS, tr("Image enregistrée"));
                }
            }
            return;
        }
        if self.compare.is_some() {
            self.compare_ui(ui);
            return;
        }
        let cache = self.vis_groups();
        let vis = &cache.vis;
        let groups = &cache.groups;
        self.stat = (vis.len(), vis.iter().map(|&i| self.items[i].size).sum());
        if let Some(v) = self.view {
            self.viewer(ui, &ctx, vis, v);
            return;
        }

        self.slideshow = false;
        if !ctx.wants_keyboard_input() && self.confirm_trash.is_none() {
            let (all, del, esc) = ctx.input(|i| {
                (i.modifiers.command && i.key_pressed(Key::A), i.key_pressed(Key::Delete), i.key_pressed(Key::Escape))
            });
            if all {
                self.sel = vis.iter().map(|&i| self.items[i].path.clone()).collect(); // Ctrl+A
            }
            if esc {
                self.sel.clear();
            }
            if del && !self.sel.is_empty() {
                self.confirm_trash = Some(self.sel.iter().cloned().collect());
                ctx.request_repaint();
            }
        }

        let mut folders: Vec<String> = self.items.iter().map(|i| i.folder.clone()).filter(|f| !f.is_empty()).collect();
        folders.sort();
        folders.dedup();
        // --- barre d'outils : recherche, tri et taille, puis filtres et actions (elle passe à la ligne si besoin) ---
        let all_picked = !vis.is_empty() && vis.iter().all(|&i| self.sel.contains(&self.items[i].path));
        let any = !vis.is_empty();
        let (mut select_all, mut deselect, mut open_dir, mut refresh) = (false, false, false, false);
        contained(ui, |ui| {
            ui.spacing_mut().slider_width = 110.0;
            ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
            // ligne 1 : recherche, tri et taille à gauche ; actions calées à droite
            split_row(
                ui,
                "gallery_bar",
                |ui| {
                    search_field(ui, &mut self.search, tr("Rechercher…"), 260.0);
                    let sort_name = match self.sort {
                        Sort::Newest => tr("Plus récentes"),
                        Sort::Oldest => tr("Plus anciennes"),
                        Sort::Name => tr("Nom"),
                        Sort::Size => tr("Taille"),
                    };
                    dropdown(ui, &ic(icons::SORT, sort_name), false, |ui| {
                        for (s, i, n) in [
                            (Sort::Newest, icons::NEWEST, tr("Plus récentes")),
                            (Sort::Oldest, icons::OLDEST, tr("Plus anciennes")),
                            (Sort::Name, icons::SORT_NAME, tr("Nom")),
                            (Sort::Size, icons::SORT_SIZE, tr("Taille")),
                        ] {
                            if menu_item(ui, i, n, self.sort == s, "").clicked() {
                                self.sort = s;
                                close_dropdown(ui);
                            }
                        }
                    });
                    // taille des miniatures : petite image … grande image
                    let weak = ui.visuals().weak_text_color();
                    ui.label(egui::RichText::new(icons::IMAGE).size(13.0).color(weak)).on_hover_text(tr("Taille des miniatures"));
                    ui.add(egui::Slider::new(&mut self.thumb_w, 120.0..=420.0).show_value(false)).on_hover_text(tr("Taille des miniatures"));
                    ui.label(egui::RichText::new(icons::IMAGE).size(18.0).color(weak));
                },
                |ui| {
                    // tout sélectionner (ou tout désélectionner si tout est déjà sélectionné)
                    if all_picked {
                        deselect = pill(ui, icons::DESELECT, tr("Tout désélectionner")).clicked();
                    } else {
                        select_all = ui
                            .add_enabled_ui(any, |ui| pill(ui, icons::SELECT_ALL, tr("Tout sélectionner")))
                            .inner
                            .on_hover_text(tr("Sélectionner toutes les captures affichées (Ctrl+A)"))
                            .clicked();
                    }
                    open_dir = pill(ui, icons::FOLDER_OPEN, tr("Ouvrir le dossier")).on_hover_text(tr("Ouvrir le dossier des captures dans l'Explorateur")).clicked();
                    refresh = pill(ui, icons::REFRESH, tr("Actualiser")).clicked();
                },
            );
            // ligne 2 : dossiers en pastilles (ou une liste s'il y en a beaucoup), favoris
            ui.horizontal_wrapped(|ui| {
                if folders.len() <= 6 {
                    if chip(ui, self.folder.is_empty(), "", tr("Tous les dossiers")).clicked() {
                        self.folder.clear();
                    }
                    for f in &folders {
                        if chip(ui, self.folder == *f, "", f).clicked() {
                            self.folder = f.clone();
                        }
                    }
                } else {
                    let shown = if self.folder.is_empty() { tr("Tous les dossiers").to_string() } else { self.folder.clone() };
                    dropdown(ui, &ic(icons::FOLDER, &shown), !self.folder.is_empty(), |ui| {
                        if menu_item(ui, icons::FOLDERS, tr("Tous les dossiers"), self.folder.is_empty(), "").clicked() {
                            self.folder.clear();
                            close_dropdown(ui);
                        }
                        ui.separator();
                        egui::ScrollArea::vertical().max_height(360.0).show(ui, |ui| {
                            for f in &folders {
                                if menu_item(ui, icons::FOLDER, f, self.folder == *f, "").clicked() {
                                    self.folder = f.clone();
                                    close_dropdown(ui);
                                }
                            }
                        });
                    });
                }
                if fav_chip(ui, self.favs_only, tr("Favoris")).clicked() {
                    self.favs_only = !self.favs_only;
                }
            });
        });
        if deselect {
            self.sel.clear();
        }
        if select_all {
            self.sel = vis.iter().map(|&i| self.items[i].path.clone()).collect();
        }
        if open_dir {
            // dossier des captures (ou le sous-dossier filtré) dans l'Explorateur
            let root = Path::new(&self.cfg.save_dir);
            let dir = if self.folder.is_empty() { root.to_path_buf() } else { root.join(&self.folder) };
            let dir = if dir.is_dir() { dir } else { root.to_path_buf() };
            let _ = std::fs::create_dir_all(&dir);
            let _ = Command::new("explorer").arg(&dir).spawn();
        }
        if refresh {
            self.items = list_items(&self.cfg.save_dir);
        }
        ui.add_space(4.0);

        // nouveau filtre, nouvelle recherche ou nouveau tri : les miniatures réapparaissent en cascade
        let sig = {
            use std::hash::{Hash, Hasher};
            let mut h = std::collections::hash_map::DefaultHasher::new();
            (&self.folder, &self.search, self.sort as u8, self.favs_only).hash(&mut h);
            h.finish()
        };
        if sig != self.grid_sig {
            if self.grid_sig != 0 {
                self.grid_at = Instant::now();
            }
            self.grid_sig = sig;
        }

        let mut acts: Vec<Act> = Vec::new();
        // pilule de sélection : elle monte du bas de la page quand on sélectionne, et redescend ensuite
        if !self.sel.is_empty() {
            self.sel_shown = self.sel.len();
        }
        let sel_k = ctx.animate_bool_with_time_and_easing(egui::Id::new("selbar"), !self.sel.is_empty(), anim::ENTER, egui::emath::easing::cubic_out);
        if sel_k > 0.01 && self.sheet_rect != Rect::NOTHING {
            let n_sel = self.sel_shown;
            let acc = self.shown_acc;
            let fg = on_accent(acc);
            let sheet = self.sheet_rect;
            let two = self.sel.len() == 2;
            egui::Area::new(egui::Id::new("selbar_area"))
                .order(egui::Order::Foreground)
                .fixed_pos(pos2(sheet.center().x, sheet.max.y - 18.0 + 30.0 * (1.0 - sel_k)))
                .pivot(Align2::CENTER_BOTTOM)
                .show(&ctx, |ui| {
                    ui.multiply_opacity(sel_k);
                    egui::Frame::none()
                        .fill(acc)
                        .rounding(26.0)
                        .shadow(egui::epaint::Shadow { offset: vec2(0.0, 12.0), blur: 32.0, spread: 0.0, color: Color32::from_black_alpha(140) })
                        .inner_margin(egui::Margin { left: 18.0, right: 6.0, top: 6.0, bottom: 6.0 })
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 6.0;
                                ui.label(egui::RichText::new(trf("{0} sélectionnée(s)", &[&n_sel])).color(fg).family(egui::FontFamily::Name("bold".into())));
                                ui.add_space(6.0);
                                let btn = |ui: &mut egui::Ui, icon: &str, label: &str| {
                                    ui.add(
                                        egui::Button::new(egui::RichText::new(ic(icon, label)).color(fg))
                                            .fill(fg.gamma_multiply(0.16))
                                            .stroke(Stroke::NONE)
                                            .rounding(18.0)
                                            .min_size(vec2(0.0, 36.0)),
                                    )
                                };
                                if two && btn(ui, icons::COMPARE, tr("Comparer")).clicked() {
                                    acts.push(Act::Compare);
                                }
                                if btn(ui, icons::COPY, tr("Copier les fichiers")).clicked() {
                                    acts.push(Act::CopyFiles);
                                }
                                if btn(ui, icons::EXPORT, tr("Exporter…")).clicked() {
                                    acts.push(Act::Export);
                                }
                                if btn(ui, icons::STAR, tr("Favori")).clicked() {
                                    for p in self.sel.clone() {
                                        acts.push(Act::Fav(p));
                                    }
                                }
                                if btn(ui, icons::TRASH, tr("Corbeille")).clicked() {
                                    acts.push(Act::AskTrash(self.sel.iter().cloned().collect()));
                                }
                                if btn(ui, icons::CLOSE, tr("Désélectionner")).clicked() {
                                    self.sel.clear();
                                }
                            });
                        });
                });
        }

        if vis.is_empty() {
            self.empty_state(ui);
            return;
        }

        let by_date = matches!(self.sort, Sort::Newest | Sort::Oldest);
        let acc = self.shown_acc;
        let ctrl = ctx.input(|i| i.modifiers.command || i.modifiers.ctrl);
        let selecting = !self.sel.is_empty();
        // la grille remplit toute la largeur : le nombre de colonnes dépend de la taille souhaitée
        let gap = 12.0_f32;
        let avail = (ui.available_width() - 28.0).max(120.0);
        let cols = ((avail + gap) / (self.thumb_w + gap)).floor().max(1.0);
        let cw = ((avail - gap * (cols - 1.0)) / cols).floor();
        let ch = (cw * 9.0 / 16.0).floor();
        let mut loads = 0;
        let mut order = 0usize; // rang de la miniature parmi celles visibles (pour la cascade)
        let since_grid = self.grid_at.elapsed().as_secs_f32();
        self.thumb_rects.clear();
        let closing_idx = self.fly.as_ref().filter(|f| f.closing && !f.fade).map(|f| f.idx);
        let now = ctx.input(|i| i.time) as f32;
        // la grille est posée dans un panneau un peu en retrait de la page
        let inset = ui.visuals().extreme_bg_color;
        egui::Frame::none().fill(inset).rounding(14.0).inner_margin(egui::Margin { left: 14.0, right: 4.0, top: 12.0, bottom: 0.0 }).show(ui, |ui| {
        egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
            ui.set_max_width(ui.available_width() - 10.0); // place pour la barre de défilement
            for (label, idxs) in groups {
                if !label.is_empty() {
                    ui.add_space(4.0);
                    // titre du jour en pastille : « Aujourd'hui · 5 »
                    let v = ui.visuals().clone();
                    egui::Frame::none().fill(v.faint_bg_color).rounding(14.0).inner_margin(egui::Margin::symmetric(12.0, 5.0)).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 6.0;
                            ui.label(bold_text(label, 14.0));
                            ui.label(egui::RichText::new(format!("· {}", idxs.len())).size(13.5).color(v.weak_text_color()));
                        });
                    });
                    ui.add_space(2.0);
                }
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = vec2(gap, gap);
                    for &i in idxs {
                        let (rect, resp) = ui.allocate_exact_size(vec2(cw, ch), Sense::click());
                        if !ui.is_rect_visible(rect) {
                            continue; // chargement paresseux : seulement ce qu'on voit
                        }
                        let path = self.items[i].path.clone();
                        let modified = self.items[i].modified;
                        let folder = self.items[i].folder.clone();
                        if !self.thumbs.contains_key(&path) && loads < 3 {
                            loads += 1;
                            let t = load_thumb(&ctx, &path);
                            self.thumbs.insert(path.clone(), t);
                            self.thumb_born.insert(path.clone(), Instant::now());
                        }
                        let fav = self.favs.contains(&path);
                        let picked = self.sel.contains(&path);
                        self.thumb_rects.insert(i, rect);
                        let hov = ui.rect_contains_pointer(rect);
                        let dt = chrono::DateTime::<chrono::Local>::from(modified);
                        let when = if by_date { dt.format("%H:%M").to_string() } else { dt.format(&format!("{} %H:%M", dfmt(false))).to_string() };
                        let name = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
                        let full_name = if folder.is_empty() { file_name(&path) } else { format!("{folder}/{}", file_name(&path)) };
                        let resp = resp.on_hover_text(full_name);
                        if hov {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }

                        // --- animations : la carte se soulève au survol, la miniature apparaît en fondu ---
                        let id = ui.id();
                        let lift = ctx.animate_bool_with_time_and_easing(id.with(("lift", i)), hov, anim::HOVER, egui::emath::easing::cubic_out);
                        let show_btns = ctx.animate_bool_with_time_and_easing(id.with(("btns", i)), hov, anim::HOVER, egui::emath::easing::cubic_out);
                        let show_pick = ctx.animate_bool_with_time_and_easing(id.with(("pickv", i)), hov || selecting, anim::HOVER, egui::emath::easing::cubic_out);
                        let check = ctx.animate_bool_with_time_and_easing(id.with(("check", i)), picked, 0.3, egui::emath::easing::back_out);
                        let fav_k = ctx.animate_bool_with_time_and_easing(id.with(("favk", i)), fav, 0.35, egui::emath::easing::back_out);
                        let fav_pop = 1.0 + 0.45 * (1.0 - (2.0 * fav_k - 1.0).abs()).max(0.0); // l'étoile grossit puis revient
                        let fade = self.thumb_born.get(&path).map(|t| ease_out((t.elapsed().as_secs_f32() / anim::ENTER).min(1.0))).unwrap_or(1.0);
                        // apparition en cascade : chaque carte arrive un peu après la précédente, en montant
                        let delay = order.min(24) as f32 * anim::STAGGER;
                        order += 1;
                        let ent = ease_out(((since_grid - delay) / anim::ENTER).clamp(0.0, 1.0));
                        if fade < 1.0 || ent < 1.0 {
                            ctx.request_repaint();
                        }
                        let op0 = ui.opacity();
                        ui.multiply_opacity(ent);
                        let rect = Rect::from_center_size(rect.center() + vec2(0.0, 18.0 * (1.0 - ent)), rect.size() * (0.94 + 0.06 * ent));
                        let rect = rect.translate(vec2(0.0, -4.0 * lift)).expand(2.0 * lift);

                        // --- miniature arrondie + bandeau d'info ---
                        let rnd = 14.0_f32;
                        let p = ui.painter();
                        if lift > 0.01 {
                            let sh = egui::epaint::Shadow {
                                offset: vec2(0.0, 10.0 * lift),
                                blur: 26.0 * lift,
                                spread: 0.0,
                                color: Color32::from_black_alpha((130.0 * lift) as u8),
                            };
                            p.add(sh.as_shape(rect, rnd));
                        }
                        p.rect_filled(rect, rnd, ui.visuals().faint_bg_color);
                        match self.thumbs.get(&path) {
                            // la capture qui revient de la visionneuse se pose ici : on n'affiche pas la miniature en double
                            Some(Some(_)) if closing_idx == Some(i) => {}
                            Some(Some(t)) => {
                                egui::Image::new(egui::load::SizedTexture::new(t.id(), rect.size()))
                                    .rounding(rnd)
                                    .tint(Color32::from_white_alpha((fade * 255.0) as u8))
                                    .paint_at(ui, rect);
                            }
                            // miniature illisible : simple icône
                            Some(None) => {
                                p.text(rect.center(), Align2::CENTER_CENTER, icons::IMAGE, FontId::proportional(28.0), ui.visuals().weak_text_color());
                            }
                            // pas encore chargée : reflet qui balaie la carte en attendant
                            None => {
                                shimmer(p, rect, now + i as f32 * 0.07, ui.visuals().dark_mode);
                                ctx.request_repaint();
                            }
                        }
                        // nom et heure dans un bandeau arrondi qui flotte en bas de la miniature
                        let bar = Rect::from_min_max(pos2(rect.min.x + 7.0, rect.max.y - 7.0 - 26.0), rect.max - vec2(7.0, 7.0));
                        p.rect_filled(bar, 9.0, Color32::from_black_alpha(165));
                        let time_rect = p.text(bar.right_center() - vec2(10.0, 0.0), Align2::RIGHT_CENTER, &when, FontId::proportional(12.5), Color32::from_rgba_unmultiplied(255, 255, 255, 190));
                        let mut job = egui::text::LayoutJob::simple_singleline(name, FontId::proportional(12.5), Color32::WHITE);
                        job.wrap.max_width = (time_rect.min.x - 10.0 - (bar.min.x + 10.0)).max(10.0);
                        job.wrap.max_rows = 1;
                        job.wrap.break_anywhere = true; // nom trop long : coupé avec « … »
                        let galley = ui.fonts(|f| f.layout_job(job));
                        p.galley(pos2(bar.min.x + 10.0, bar.center().y - galley.size().y / 2.0), galley, Color32::WHITE);
                        if picked {
                            p.rect_stroke(rect, rnd, Stroke::new(3.0_f32, acc));
                        } else if lift > 0.01 {
                            p.rect_stroke(rect, rnd, Stroke::new(2.0_f32, acc.gamma_multiply(lift)));
                        }

                        // --- boutons rapides au survol ---
                        let (bg_n, bg_h) = (col(24, 26, 31), col(58, 62, 72));
                        let bs = 28.0_f32;
                        let slide = vec2(0.0, -6.0 * (1.0 - show_btns));
                        let r_trash = Rect::from_min_size(pos2(rect.max.x - 8.0 - bs, rect.min.y + 8.0), vec2(bs, bs)).translate(slide);
                        let r_copy = r_trash.translate(vec2(-(bs + 6.0), 0.0));
                        let r_fav = r_copy.translate(vec2(-(bs + 6.0), 0.0));
                        let r_pick = Rect::from_min_size(rect.min + vec2(8.0, 8.0) + vec2(0.0, -6.0 * (1.0 - show_pick)), vec2(bs, bs));
                        let mut hit = false;
                        // cercle de sélection : apparaît en fondu, la coche « rebondit » quand on sélectionne
                        if show_pick > 0.01 {
                            let mut pp = p.clone();
                            pp.multiply_opacity(show_pick);
                            let rp = ui.interact(r_pick, id.with(("pick", i)), Sense::click());
                            if rp.clicked() {
                                acts.push(Act::Pick(path.clone()));
                                hit = true;
                            }
                            pp.circle_filled(r_pick.center(), 13.0, Color32::from_black_alpha(if rp.hovered() { 200 } else { 120 }));
                            pp.circle_stroke(r_pick.center(), 13.0, Stroke::new(1.5_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 190)));
                            if check > 0.01 {
                                pp.circle_filled(r_pick.center(), 13.0 * check.min(1.15), acc);
                                if check > 0.5 {
                                    icon_check(&pp, r_pick.center(), on_accent(acc));
                                }
                            }
                        }
                        let mut bp = p.clone();
                        bp.multiply_opacity(show_btns);
                        let p = &bp; // les boutons rapides suivent le fondu du survol
                        if show_btns > 0.01 {
                            let rt = ui.interact(r_trash, id.with(("trash", i)), Sense::click()).on_hover_text(tr("Mettre à la corbeille"));
                            let rc = ui.interact(r_copy, id.with(("copy", i)), Sense::click()).on_hover_text(tr("Copier l'image"));
                            let rf = ui.interact(r_fav, id.with(("fav", i)), Sense::click()).on_hover_text(tr("Favori"));
                            if rt.clicked() {
                                acts.push(Act::AskTrash(vec![path.clone()]));
                                hit = true;
                            }
                            if rc.clicked() {
                                acts.push(Act::Copy(path.clone()));
                                hit = true;
                            }
                            if rf.clicked() {
                                acts.push(Act::Fav(path.clone()));
                                hit = true;
                            }
                            let glyph = FontId::proportional(15.0);
                            p.circle_filled(r_trash.center(), bs / 2.0, if rt.hovered() { col(185, 45, 45) } else { bg_n });
                            p.text(r_trash.center(), Align2::CENTER_CENTER, icons::TRASH, glyph.clone(), Color32::WHITE);
                            p.circle_filled(r_copy.center(), bs / 2.0, if rc.hovered() { bg_h } else { bg_n });
                            p.text(r_copy.center(), Align2::CENTER_CENTER, icons::COPY, glyph, Color32::WHITE);
                            p.circle_filled(r_fav.center(), bs / 2.0, if rf.hovered() { bg_h } else { bg_n });
                            p.text(
                                r_fav.center(),
                                Align2::CENTER_CENTER,
                                "★",
                                FontId::proportional(16.0 * fav_pop),
                                if fav { Color32::GOLD } else { Color32::WHITE },
                            );
                        }
                        if fav && show_btns < 0.99 {
                            let mut fp = ui.painter().clone();
                            fp.multiply_opacity(1.0 - show_btns);
                            fp.text(rect.right_top() + vec2(-10.0, 8.0), Align2::RIGHT_TOP, "★", FontId::proportional(20.0 * fav_pop), Color32::GOLD);
                        }

                        ui.set_opacity(op0);
                        resp.context_menu(|ui| {
                            ui.spacing_mut().item_spacing.y = 2.0;
                            let fav_label = if fav { tr("Retirer des favoris") } else { tr("★ Ajouter aux favoris").trim_start_matches("★ ") };
                            // trois groupes : l'essentiel, le fichier, puis la corbeille (en rouge)
                            let groups: [Vec<(&str, &str, Act)>; 2] = [
                                vec![
                                    (icons::COPY, tr("Copier l'image"), Act::Copy(path.clone())),
                                    (if fav { icons::STAR_OFF } else { icons::STAR }, fav_label, Act::Fav(path.clone())),
                                    (icons::RENAME, tr("Renommer…"), Act::Rename(path.clone())),
                                    (icons::ANNOTATE, tr("Annoter (éditeur intégré)"), Act::Annotate(path.clone())),
                                    (icons::PIN, tr("Épingler à l'écran"), Act::Pin(path.clone())),
                                ],
                                vec![
                                    (icons::FOLDER, tr("Copier le fichier"), Act::CopyFile(path.clone())),
                                    (icons::LINK, tr("Copier le chemin"), Act::Path(path.clone())),
                                    (icons::EDIT_EXT, tr("Modifier dans l'éditeur"), Act::Edit(path.clone())),
                                    (icons::FOLDER_OPEN, tr("Afficher dans le dossier"), Act::Reveal(path.clone())),
                                ],
                            ];
                            for (n, group) in groups.into_iter().enumerate() {
                                if n > 0 {
                                    ui.separator();
                                }
                                for (icon, label, act) in group {
                                    if menu_item(ui, icon, label, false, "").clicked() {
                                        acts.push(act);
                                        ui.close_menu();
                                    }
                                }
                            }
                            ui.separator();
                            if menu_item_colored(ui, icons::TRASH, tr("Mettre à la corbeille"), false, "", Some(col(239, 68, 68))).clicked() {
                                acts.push(Act::AskTrash(vec![path.clone()]));
                                ui.close_menu();
                            }
                        });
                        if resp.clicked() && !hit {
                            acts.push(if ctrl { Act::Pick(path.clone()) } else { Act::Open(i) });
                        }
                    }
                });
            }
            ui.add_space(80.0); // la pilule de sélection ne cache jamais la dernière rangée
        });
        });
        if loads > 0 {
            ctx.request_repaint();
        }
        for a in acts {
            match a {
                Act::Open(i) => {
                    self.fly_from = self.thumb_rects.get(&i).copied();
                    self.view = Some(i);
                    self.view_dir = 0.0;
                }
                Act::Pick(p) => {
                    if !self.sel.remove(&p) {
                        self.sel.insert(p);
                    }
                }
                Act::Fav(p) => self.toggle_fav(p),
                Act::Copy(p) => {
                    if let Ok(i) = image::open(&p) {
                        copy_to_clipboard(&i.to_rgba8());
                        self.status = ic(icons::COPY, tr("Image copiée dans le presse-papiers"));
                    }
                }
                Act::Reveal(p) => {
                    let _ = Command::new("explorer").arg("/select,").arg(&p).spawn();
                }
                Act::AskTrash(v) => self.confirm_trash = Some(v),
                Act::Path(p) => {
                    copy_text(&p.display().to_string());
                    self.status = ic(icons::LINK, tr("Chemin copié"));
                }
                Act::Edit(p) => self.edit(&p),
                Act::Rename(p) => {
                    let stem = p.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
                    self.rename_dlg = Some((p, stem));
                }
                Act::Pin(p) => {
                    let s = p.display().to_string();
                    let _ = spawn_self(&["--pin", s.as_str()]);
                }
                Act::Annotate(p) => self.editor = Editor::open(&ctx, &p),
                Act::CopyFile(p) => {
                    copy_files(&[p]);
                    self.status = ic(icons::COPY, tr("Fichier copié : Ctrl+V dans l'Explorateur, un mail…"));
                }
                Act::CopyFiles => {
                    let v: Vec<PathBuf> = self.sel.iter().cloned().collect();
                    copy_files(&v);
                    self.status = ic(icons::COPY, &trf("{0} fichier(s) copié(s)", &[&v.len()]));
                }
                Act::Export => {
                    if let Some(d) = rfd::FileDialog::new().pick_folder() {
                        let mut n = 0;
                        for p in &self.sel {
                            if let Some(name) = p.file_name() {
                                if std::fs::copy(p, d.join(name)).is_ok() {
                                    n += 1;
                                }
                            }
                        }
                        self.status = ic(icons::EXPORT, &trf("{0} fichier(s) exporté(s)", &[&n]));
                    }
                }
                Act::Compare => {
                    let v: Vec<PathBuf> = self.sel.iter().cloned().collect();
                    if v.len() == 2 {
                        let load = |p: &PathBuf| image::open(p).ok().map(|i| tex(&ctx, "cmp", &i.to_rgba8()));
                        if let (Some(a), Some(b)) = (load(&v[0]), load(&v[1])) {
                            self.compare = Some(Cmp { a, b, na: file_name(&v[0]), nb: file_name(&v[1]), split: 0.5 });
                        }
                    }
                }
            }
        }
    }
}

/// Reflet clair qui balaie une carte vide (miniature en cours de chargement).
fn shimmer(p: &egui::Painter, rect: Rect, t: f32, dark: bool) {
    let w = rect.width() * 0.55;
    let x = rect.min.x - w + (t * 0.8).rem_euclid(1.6) / 1.6 * (rect.width() + 2.0 * w);
    let a = if dark { 22 } else { 70 };
    let clear = Color32::TRANSPARENT;
    let shine = Color32::from_white_alpha(a);
    let mut m = egui::Mesh::default();
    let (y0, y1) = (rect.min.y, rect.max.y);
    for (dx, c) in [(0.0, clear), (w / 2.0, shine), (w, clear)] {
        m.colored_vertex(pos2(x + dx, y0), c);
        m.colored_vertex(pos2(x + dx, y1), c);
    }
    m.add_triangle(0, 1, 2);
    m.add_triangle(1, 3, 2);
    m.add_triangle(2, 3, 4);
    m.add_triangle(3, 5, 4);
    p.with_clip_rect(rect.shrink(1.0).intersect(p.clip_rect())).add(m);
}

impl App {
    /// Écrans vides : aucune capture, aucun résultat de recherche, aucun favori, dossier vide.
    fn empty_state(&mut self, ui: &mut egui::Ui) {
        let (icon, title, text, button): (&str, String, String, String) = if self.items.is_empty() {
            (
                icons::CAMERA,
                tr("Aucune capture pour l'instant").to_string(),
                trf("Appuie sur {0} pour capturer l'écran, ou {1} pour une zone : tes captures apparaîtront ici.", &[&self.cfg.hotkey, &self.cfg.hotkey_region]),
                ic(icons::FOLDER_OPEN, tr("Ouvrir le dossier")),
            )
        } else if !self.search.is_empty() {
            (
                icons::SEARCH_X,
                trf("Aucun résultat pour « {0} »", &[&self.search]),
                tr("Essaie un autre mot, ou cherche par date (par exemple 2026-11).").to_string(),
                ic(icons::CLOSE, tr("Effacer la recherche")),
            )
        } else if self.favs_only {
            (
                icons::STAR_OFF,
                tr("Aucun favori").to_string(),
                tr("Survole une capture et clique sur l'étoile, ou appuie sur F dans la visionneuse.").to_string(),
                ic(icons::GALLERY, tr("Afficher toutes les captures")),
            )
        } else {
            (icons::FOLDER, tr("Ce dossier est vide").to_string(), String::new(), ic(icons::FOLDER, tr("Tous les dossiers")))
        };
        let clicked = empty_view(ui, self.grid_at, self.shown_acc, icon, &title, &text, &button);
        if clicked {
            if self.items.is_empty() {
                let _ = std::fs::create_dir_all(&self.cfg.save_dir);
                let _ = Command::new("explorer").arg(&self.cfg.save_dir).spawn();
            } else if !self.search.is_empty() {
                self.search.clear();
            } else if self.favs_only {
                self.favs_only = false;
            } else {
                self.folder.clear();
            }
        }
    }
}
