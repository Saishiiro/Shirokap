// Page des paramètres (par rubriques) et réinitialisations.
use super::*;

impl App {
    // ----- paramètres -----
    pub(super) fn settings_ui(&mut self, ui: &mut egui::Ui) {
        // fenêtre étroite : les rubriques se réduisent à leurs tuiles (leur nom s'affiche au survol)
        let compact = ui.available_width() < 760.0;
        egui::SidePanel::left("set_nav")
            .exact_width(if compact { 48.0 } else { 210.0 })
            .resizable(false)
            .show_separator_line(false)
            .frame(egui::Frame::none())
            .show_inside(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 2.0;
                for (n, (c, i, name)) in [
                    (SetCat::Shortcuts, icons::KEYBOARD, tr("Raccourcis")),
                    (SetCat::Capture, icons::CAMERA, tr("Capture")),
                    (SetCat::Files, icons::FOLDER, tr("Fichiers")),
                    (SetCat::Look, icons::PALETTE, tr("Apparence")),
                    (SetCat::Privacy, icons::SHIELD, tr("Confidentialité")),
                    (SetCat::System, icons::SYSTEM, tr("Système")),
                ]
                .into_iter()
                .enumerate()
                {
                    if compact {
                        cat_tile(ui, &mut self.set_cat, c, i, name, cat_tint(n));
                    } else {
                        cat_item(ui, &mut self.set_cat, c, i, name, cat_tint(n));
                    }
                }
                ui.add_space(14.0);
                // remet toutes les rubriques à zéro (rien n'est appliqué avant « Enregistrer »)
                let text = if compact { icons::RESET.to_string() } else { ic(icons::RESET, tr("Tout réinitialiser")) };
                let reset = egui::Button::new(egui::RichText::new(text).size(13.5))
                    .fill(Color32::TRANSPARENT)
                    .stroke(Stroke::NONE)
                    .rounding(18.0);
                if ui
                    .add(reset)
                    .on_hover_ui(|ui| {
                        if compact {
                            ui.strong(tr("Tout réinitialiser"));
                        }
                        ui.label(tr("Remet TOUS les réglages de TOUTES les rubriques à leur valeur d'origine. Rien n'est appliqué tant que tu n'as pas cliqué sur « Enregistrer » (« Annuler » permet de revenir en arrière)."));
                    })
                    .clicked()
                {
                    self.cfg = Config::default();
                }
            });
        // changement de rubrique : le contenu arrive en fondu depuis la droite
        if self.set_cat != self.shown_cat {
            self.shown_cat = self.set_cat;
            self.cat_at = Instant::now();
        }
        let k = ease_out((self.cat_at.elapsed().as_secs_f32() / anim::PAGE).min(1.0));
        if k < 1.0 {
            ui.ctx().request_repaint();
        }
        let pad = egui::Margin { left: if compact { 10.0 } else { 18.0 } + 28.0 * (1.0 - k), right: 0.0, top: 0.0, bottom: 0.0 };
        egui::Frame::none().inner_margin(pad).show(ui, |ui| {
            ui.multiply_opacity(k);
            egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
                // marge à droite pour la barre de défilement, qui flotte par-dessus le contenu
                ui.set_max_width(ui.available_width() - 14.0);
                match self.set_cat {
                    SetCat::Shortcuts => self.set_shortcuts(ui),
                    SetCat::Capture => self.set_capture(ui),
                    SetCat::Files => self.set_files(ui),
                    SetCat::Look => self.set_look(ui),
                    SetCat::Privacy => self.set_privacy(ui),
                    SetCat::System => self.set_system(ui),
                }
                ui.add_space(70.0); // place pour la pilule d'enregistrement, qui flotte en bas
            });
        });
    }

    /// Remet à leur valeur d'origine les réglages d'une rubrique (à enregistrer ensuite avec le bouton du bas).
    pub(super) fn reset_cat(&mut self, cat: SetCat) {
        let d = Config::default();
        let c = &mut self.cfg;
        match cat {
            SetCat::Shortcuts => {
                c.hotkey = d.hotkey;
                c.hotkey_region = d.hotkey_region;
                c.hotkey_window = d.hotkey_window;
                c.hotkey_last_region = d.hotkey_last_region;
                c.hotkey_gallery = d.hotkey_gallery;
                c.hotkey_region_clip = d.hotkey_region_clip;
                c.hotkey_picker = d.hotkey_picker;
            }
            SetCat::Capture => {
                c.monitor_mode = d.monitor_mode;
                c.target_width = d.target_width;
                c.delay_secs = d.delay_secs;
                c.interval_secs = d.interval_secs;
                c.clipboard = d.clipboard;
                c.flash = d.flash;
                c.sound = d.sound;
            }
            SetCat::Files => {
                c.save_dir = d.save_dir;
                c.name_template = d.name_template;
                c.format = d.format;
                c.jpg_quality = d.jpg_quality;
                c.png_level = d.png_level;
                c.folder_per_window = d.folder_per_window;
                c.cleanup_days = d.cleanup_days;
                c.editor = d.editor;
            }
            SetCat::Look => {
                c.theme = d.theme;
                c.accent = d.accent;
                c.accent_auto = d.accent_auto;
                c.tray_menu = d.tray_menu;
            }
            SetCat::Privacy => c.blocklist = d.blocklist,
            SetCat::System => c.language = d.language,
        }
        self.status = ic(icons::RESET, tr("Rubrique réinitialisée : pense à enregistrer"));
    }

    /// Bouton discret en bas de chaque rubrique : remet seulement cette rubrique à zéro.
    pub(super) fn reset_row(&mut self, ui: &mut egui::Ui, cat: SetCat) {
        if ghost(ui, icons::RESET, tr("Rétablir les valeurs par défaut de cette rubrique"))
            .on_hover_text(tr("Remet uniquement les réglages de cette page à leur valeur d'origine. Rien n'est appliqué tant que tu n'as pas cliqué sur « Enregistrer »."))
            .clicked()
        {
            self.reset_cat(cat);
        }
        ui.add_space(8.0);
    }

    pub(super) fn set_shortcuts(&mut self, ui: &mut egui::Ui) {
        page_head(ui, icons::KEYBOARD, tr("Raccourcis"), tr("Des touches ou combinaisons qui fonctionnent partout dans Windows. Écris-les dans le champ, ou clique sur « Enregistrer » puis appuie sur les touches."));
        let rec = &mut self.recording;
        let c = &mut self.cfg;
        group(ui, tr("Capture"), |ui, rows| {
            hotkey_row(ui, rows, tr("Capture"), &mut c.hotkey, rec, 0);
            hotkey_row(ui, rows, tr("Zone (sélection)"), &mut c.hotkey_region, rec, 1);
            hotkey_row(ui, rows, tr("Fenêtre active"), &mut c.hotkey_window, rec, 2);
            hotkey_row(ui, rows, tr("Dernière zone"), &mut c.hotkey_last_region, rec, 3);
            hotkey_row(ui, rows, tr("Zone → presse-papiers"), &mut c.hotkey_region_clip, rec, 5);
            hotkey_row(ui, rows, tr("Pipette (couleur)"), &mut c.hotkey_picker, rec, 6);
        });
        group(ui, tr("Application"), |ui, rows| {
            hotkey_row(ui, rows, tr("Ouvrir la galerie"), &mut c.hotkey_gallery, rec, 4);
        });
        ui.horizontal_wrapped(|ui| {
            ui.add_space(4.0);
            ui.label(egui::RichText::new(icons::INFO).color(ui.visuals().weak_text_color()));
            ui.label(egui::RichText::new(tr("Une touche seule fonctionne (F7, Pause, P…), tout comme une combinaison (Ctrl, Alt, Shift ou Win + une touche). Laisse un champ vide pour désactiver le raccourci.")).size(13.0).color(ui.visuals().weak_text_color()));
        });
        ui.add_space(18.0);
        group(ui, tr("Dans la galerie"), |ui, rows| {
            for (icon, l) in [
                (icons::GALLERY, tr("Clic : ouvrir  ·  Ctrl+clic : sélectionner  ·  clic droit : menu  ·  glisser des images dessus : les importer")),
                (icons::KEYBOARD, tr("Ctrl+A : tout sélectionner  ·  Suppr : corbeille  ·  Échap : désélectionner  ·  Ctrl+B : menu latéral  ·  Ctrl+K : palette de commandes")),
                (icons::IMAGE, tr("Visionneuse : ← → naviguer  ·  F favori  ·  Espace diaporama  ·  F11 plein écran  ·  molette zoom")),
                (icons::ZOOM_IN, tr("Zoom : molette  ·  + et − par paliers  ·  0 ajuster  ·  1 taille réelle  ·  double-clic (visionneuse) : ajuster / taille réelle")),
                (icons::PEN, tr("Éditeur : Ctrl+Z annuler  ·  Ctrl+Y rétablir  ·  clic molette ou Espace + glisser : déplacer l'image")),
                (icons::PIN, tr("Capture épinglée : glisser pour déplacer  ·  molette pour la taille  ·  clic droit ou Échap pour fermer")),
                (icons::MENU, tr("Icône du tray : clic gauche = galerie  ·  clic molette = capture  ·  clic droit = menu")),
            ] {
                row_info(ui, rows, icon, l);
            }
        });
        self.reset_row(ui, SetCat::Shortcuts);
    }

    pub(super) fn set_capture(&mut self, ui: &mut egui::Ui) {
        page_head(ui, icons::CAMERA, tr("Capture"), tr("Comment l'écran est capturé, et ce qui se passe juste après."));
        let c = &mut self.cfg;
        group(ui, tr("Source"), |ui, rows| {
            row(ui, rows, tr("Écran à capturer"), tr("L'écran utilisé par le raccourci de capture."), |ui| {
                segmented(
                    ui,
                    &mut c.monitor_mode,
                    &[("primary".to_string(), tr("Principal")), ("cursor".to_string(), tr("Sous la souris")), ("all".to_string(), tr("Tous les écrans"))],
                );
            });
            row(
                ui,
                rows,
                tr("Résolution de sortie"),
                tr("« Native » garde la résolution réelle de l'écran (vrai 4K sur un écran 4K). Choisir 4K sur un écran plus petit agrandit l'image, sans détail en plus."),
                |ui| {
                    let label = |w: u32| match w {
                        0 => tr("Native (celle de l'écran)"),
                        3840 => tr("4K · 3840 px de large"),
                        2560 => "1440p · 2560 px",
                        1920 => "1080p · 1920 px",
                        _ => "720p · 1280 px",
                    };
                    egui::ComboBox::new("res", "").width(220.0).selected_text(label(c.target_width)).show_ui(ui, |ui| {
                        for w in [0u32, 3840, 2560, 1920, 1280] {
                            ui.selectable_value(&mut c.target_width, w, label(w));
                        }
                    });
                },
            );
            row(ui, rows, tr("Délai avant capture"), tr("Attend quelques secondes avant de capturer : pratique pour ouvrir un menu ou changer de fenêtre."), |ui| {
                stepper(ui, "delay", &mut c.delay_secs, 0..=60, 1, "s");
            });
        });
        group(ui, "Timelapse", |ui, rows| {
            row(ui, rows, tr("Intervalle"), tr("Une capture toutes les N secondes quand le timelapse est activé depuis le menu de l'icône du tray."), |ui| {
                stepper(ui, "interval", &mut c.interval_secs, 5..=3600, 5, "s");
            });
        });
        group(ui, tr("Après la capture"), |ui, rows| {
            toggle_row(ui, rows, tr("Copier dans le presse-papiers"), tr("L'image est prête à être collée (Ctrl+V) dans un message, un document…"), &mut c.clipboard);
            toggle_row(ui, rows, tr("Clignotement de l'icône"), tr("L'icône du tray devient verte un instant pour confirmer la capture."), &mut c.flash);
            toggle_row(ui, rows, tr("Bip sonore"), tr("Joue un petit son à chaque capture (assets/beep.wav)."), &mut c.sound);
        });
        self.reset_row(ui, SetCat::Capture);
    }

    pub(super) fn set_files(&mut self, ui: &mut egui::Ui) {
        page_head(ui, icons::FOLDER, tr("Fichiers"), tr("Où et comment les captures sont enregistrées."));
        let c = &mut self.cfg;
        group(ui, tr("Enregistrement"), |ui, rows| {
            row_below(ui, rows, tr("Dossier des captures"), tr("Le dossier est créé automatiquement s'il n'existe pas."), |ui| {
                ui.horizontal(|ui| {
                    let w = (ui.available_width() - 140.0).max(160.0);
                    field(ui, &mut c.save_dir, "", w);
                    if pill(ui, icons::FOLDER_OPEN, tr("Parcourir…")).clicked() {
                        if let Some(p) = rfd::FileDialog::new().pick_folder() {
                            c.save_dir = p.display().to_string();
                        }
                    }
                });
            });
            row_below(ui, rows, tr("Nom des fichiers"), tr("Clique sur une variable pour l'ajouter au nom."), |ui| {
                field(ui, &mut c.name_template, "{date}_{time}", ui.available_width());
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
                    for (var, what) in [
                        ("{window}", tr("fenêtre")),
                        ("{date}", tr("date")),
                        ("{time}", tr("heure")),
                        ("{res}", tr("résolution")),
                        ("{n}", tr("numéro")),
                    ] {
                        let acc = accent_of(ui.visuals());
                        let mut job = egui::text::LayoutJob::default();
                        let f = FontId::monospace(12.5);
                        job.append(var, 0.0, egui::TextFormat { font_id: f, color: acc.lerp_to_gamma(ui.visuals().text_color(), 0.3), ..Default::default() });
                        job.append(what, 6.0, egui::TextFormat { font_id: FontId::proportional(12.5), color: ui.visuals().weak_text_color(), ..Default::default() });
                        if ui.add(egui::Button::new(job).rounding(15.0).min_size(vec2(0.0, 30.0)).stroke(Stroke::NONE)).clicked() {
                            let t = &mut c.name_template;
                            if !t.is_empty() && !t.ends_with(['_', '-', ' ']) {
                                t.push('_');
                            }
                            t.push_str(var);
                        }
                    }
                });
                // aperçu du nom avec des valeurs d'exemple
                let now = chrono::Local::now();
                let ext = if c.format == "jpg" { "jpg" } else { "png" };
                let name = c
                    .name_template
                    .replace("{window}", "Firefox")
                    .replace("{date}", &now.format("%Y-%m-%d").to_string())
                    .replace("{time}", &now.format("%Hh%Mm%Ss").to_string())
                    .replace("{res}", "2560x1440")
                    .replace("{n}", "0001");
                let name = if name.trim().is_empty() { "capture".to_string() } else { name };
                ui.horizontal_wrapped(|ui| {
                    let weak = ui.visuals().weak_text_color();
                    ui.label(egui::RichText::new(icons::IMAGE).color(weak));
                    ui.label(egui::RichText::new(tr("Aperçu :")).size(13.0).color(weak));
                    ui.label(egui::RichText::new(format!("{name}.{ext}")).monospace().size(12.5));
                });
            });
            row(ui, rows, tr("Format"), tr("PNG garde une qualité parfaite ; JPEG est plus léger."), |ui| {
                segmented(ui, &mut c.format, &[("png".to_string(), "PNG"), ("jpg".to_string(), "JPEG")]);
            });
            if c.format == "jpg" {
                row(ui, rows, tr("Qualité JPEG"), tr("Plus elle est haute, plus l'image est fidèle (et lourde)."), |ui| {
                    ui.spacing_mut().slider_width = 160.0;
                    ui.add(egui::Slider::new(&mut c.jpg_quality, 50..=100));
                });
            } else {
                row(ui, rows, tr("Compression PNG"), tr("Maximale : fichiers plus petits, enregistrement un peu plus lent. La qualité reste parfaite."), |ui| {
                    segmented(ui, &mut c.png_level, &[(0u8, tr("Rapide")), (1u8, tr("Normale")), (2u8, tr("Maximale"))]);
                });
            }
            toggle_row(ui, rows, tr("Un sous-dossier par fenêtre ou jeu"), tr("Range chaque capture dans un dossier au nom de la fenêtre active."), &mut c.folder_per_window);
        });
        group(ui, tr("Rangement automatique"), |ui, rows| {
            row(
                ui,
                rows,
                tr("Vider les vieilles captures"),
                tr("Met à la corbeille les captures plus anciennes que ce délai. 0 = désactivé. Les favoris sont toujours conservés, et la corbeille Windows permet de tout récupérer."),
                |ui| {
                    stepper(ui, "cleanup", &mut c.cleanup_days, 0..=9999, 1, tr(" jours").trim());
                },
            );
        });
        group(ui, tr("Éditeur externe"), |ui, rows| {
            row_below(ui, rows, tr("Programme d'édition"), tr("Utilisé par le bouton « Modifier dans l'éditeur externe » (Paint par défaut)."), |ui| {
                ui.horizontal(|ui| {
                    let w = (ui.available_width() - 140.0).max(160.0);
                    field(ui, &mut c.editor, "mspaint", w);
                    if pill(ui, icons::FOLDER_OPEN, tr("Parcourir…")).clicked() {
                        if let Some(p) = rfd::FileDialog::new().add_filter("exe", &["exe"]).pick_file() {
                            c.editor = p.display().to_string();
                        }
                    }
                });
            });
        });
        self.reset_row(ui, SetCat::Files);
    }

    pub(super) fn set_look(&mut self, ui: &mut egui::Ui) {
        page_head(ui, icons::PALETTE, tr("Apparence"), tr("Le thème et la couleur de l'interface. Les changements sont visibles tout de suite."));
        // les cartes de thème puis les pastilles d'accent apparaissent en cascade à l'arrivée sur la page
        let since = self.tab_at.elapsed().min(self.cat_at.elapsed()).as_secs_f32();
        let appear = |n: usize| ease_out(((since - 0.1 - n as f32 * 0.035) / 0.45).clamp(0.0, 1.0));
        if appear(THEMES.len() + ACCENTS.len() + 1) < 1.0 {
            ui.ctx().request_repaint();
        }
        let c = &mut self.cfg;
        group(ui, tr("Thème"), |ui, rows| {
            row_below(ui, rows, tr("Thème"), tr("Choisis l'ambiance de l'interface : le changement est immédiat."), |ui| {
                let fixed = c.accent;
                let auto = c.accent_auto;
                ui.horizontal_wrapped(|ui| {
                    for (n, (key, name)) in THEMES.iter().enumerate() {
                        let pal = palette(key);
                        // en mode « Auto », chaque vignette montre la couleur d'accent de son propre thème
                        let a = if auto { theme_accent(key) } else { fixed };
                        if theme_card(ui, c.theme == *key, tr(name), &pal, col(a[0], a[1], a[2]), appear(n)).clicked() {
                            c.theme = key.to_string();
                        }
                    }
                });
            });
            row_below(ui, rows, tr("Couleur d'accent"), tr("Boutons principaux, sélection, barres et liseré du menu. Choisis une couleur rapide ou une couleur libre. « Auto » prend la couleur propre à chaque thème."), |ui| {
                ui.horizontal_wrapped(|ui| {
                    let ctx = ui.ctx().clone();
                    let now = ctx.input(|i| i.time);
                    let rip_id = egui::Id::new("accent_ripple");
                    let ripple = ctx.data(|d| d.get_temp::<(usize, f64)>(rip_id));
                    // bouton « Auto » : la couleur suit le thème. Pastille aux couleurs du thème actuel + libellé,
                    // dans une capsule pour le distinguer des couleurs fixes
                    {
                        let ta = theme_accent(&c.theme);
                        let tcol = col(ta[0], ta[1], ta[2]);
                        let (r, resp) = ui.allocate_exact_size(vec2(84.0, 28.0), Sense::click());
                        let resp = resp.on_hover_text(tr("Couleur du thème : l'accent suit le thème choisi"));
                        if resp.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        let on = c.accent_auto;
                        let hk = ctx.animate_bool_with_time_and_easing(resp.id.with("hov"), resp.hovered(), anim::HOVER, egui::emath::easing::cubic_out);
                        let sk = ctx.animate_bool_with_time_and_easing(resp.id.with("sel"), on, 0.4, egui::emath::easing::back_out);
                        let k = appear(THEMES.len());
                        let mut p = ui.painter().clone();
                        p.multiply_opacity(k);
                        let r = r.translate(vec2(0.0, 10.0 * (1.0 - k))).expand(1.5 * hk);
                        let v = ui.visuals();
                        let txt = v.text_color();
                        p.rect_filled(r, 14.0, v.widgets.inactive.bg_fill.lerp_to_gamma(v.widgets.hovered.bg_fill, hk));
                        p.rect_stroke(r, 14.0, Stroke::new(1.0 + 1.0 * sk.clamp(0.0, 1.0), v.widgets.inactive.bg_stroke.color.lerp_to_gamma(tcol, sk.clamp(0.0, 1.0))));
                        // pastille en dégradé circulaire de quelques couleurs de thèmes, centre = couleur du thème actuel
                        let cc = pos2(r.min.x + 15.0, r.center().y);
                        let spin = if resp.hovered() { ctx.input(|i| i.time) as f32 * 1.2 } else { 0.0 }; // tourne au survol
                        let segs = [theme_accent("ocean"), theme_accent("neon"), theme_accent("ruby"), theme_accent("mocha"), theme_accent("forest"), theme_accent("midnight")];
                        for (j, sc) in segs.iter().enumerate() {
                            let a0 = j as f32 / segs.len() as f32 * std::f32::consts::TAU;
                            let a1 = (j + 1) as f32 / segs.len() as f32 * std::f32::consts::TAU;
                            let pts: Vec<Pos2> = std::iter::once(cc)
                                .chain((0..=6).map(|t| {
                                    let a = a0 + (a1 - a0) * t as f32 / 6.0 + spin;
                                    cc + vec2(a.cos(), a.sin()) * 9.0
                                }))
                                .collect();
                            p.add(egui::Shape::convex_polygon(pts, col(sc[0], sc[1], sc[2]), Stroke::NONE));
                        }
                        p.circle_filled(cc, 5.0, tcol);
                        p.circle_stroke(cc, 5.0, Stroke::new(1.5_f32, v.widgets.inactive.bg_fill));
                        p.text(pos2(cc.x + 14.0, r.center().y), Align2::LEFT_CENTER, tr("Auto|accent"), FontId::new(14.0, egui::FontFamily::Name("bold".into())), txt);
                        if resp.hovered() {
                            ctx.request_repaint();
                        }
                        if let Some((rn, t0)) = ripple {
                            let e = ((now - t0) as f32 / 0.6).clamp(0.0, 1.0);
                            if rn == usize::MAX && e < 1.0 {
                                p.rect_stroke(r.expand(10.0 * ease_out(e)), 14.0 + 10.0 * e, Stroke::new(2.5 * (1.0 - e), tcol));
                                ctx.request_repaint();
                            }
                        }
                        if resp.clicked() {
                            c.accent_auto = true;
                            ctx.data_mut(|d| d.insert_temp(rip_id, (usize::MAX, now)));
                        }
                        ui.add_space(10.0);
                    }
                    for (n, a) in ACCENTS.iter().enumerate() {
                        let (r, resp) = ui.allocate_exact_size(vec2(28.0, 28.0), Sense::click());
                        if resp.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        let chosen = !c.accent_auto && c.accent == *a;
                        // survol : la pastille grossit ; choix : l'anneau se referme autour en rebondissant
                        let hk = ctx.animate_bool_with_time_and_easing(resp.id.with("hov"), resp.hovered(), anim::HOVER, egui::emath::easing::back_out);
                        let sk = ctx.animate_bool_with_time_and_easing(resp.id.with("sel"), chosen, 0.4, egui::emath::easing::back_out);
                        let k = appear(THEMES.len() + 1 + n);
                        let ring = ui.visuals().text_color();
                        let color = col(a[0], a[1], a[2]);
                        let mut p = ui.painter().clone();
                        p.multiply_opacity(k);
                        let center = r.center() + vec2(0.0, 10.0 * (1.0 - k));
                        p.circle_filled(center, (10.0 + 2.0 * hk) * (0.4 + 0.6 * k), color);
                        if sk > 0.01 {
                            p.circle_stroke(center, 10.0 + 3.5 * sk, Stroke::new(2.0 * sk.min(1.0), ring));
                        } else if hk > 0.01 {
                            p.circle_stroke(center, 12.5, Stroke::new(1.2_f32, ring.gamma_multiply(hk.min(1.0))));
                        }
                        // onde de la couleur choisie
                        if let Some((rn, t0)) = ripple {
                            let e = ((now - t0) as f32 / 0.6).clamp(0.0, 1.0);
                            if rn == n && e < 1.0 {
                                p.circle_stroke(center, 13.0 + 16.0 * ease_out(e), Stroke::new(2.5 * (1.0 - e), color));
                                ctx.request_repaint();
                            }
                        }
                        if resp.clicked() {
                            c.accent = *a;
                            c.accent_auto = false;
                            ctx.data_mut(|d| d.insert_temp(rip_id, (n, now)));
                        }
                    }
                    ui.add_space(8.0);
                    // couleur libre : elle montre la couleur affichée ; la modifier repasse en couleur fixe
                    let mut free = c.accent_rgb();
                    color_button(ui, "accent", &mut free);
                    if free != c.accent_rgb() {
                        c.accent = free;
                        c.accent_auto = false;
                    }
                });
            });
        });
        group(ui, tr("Menu de l'icône du tray"), |ui, rows| {
            row(
                ui,
                rows,
                tr("Style du menu"),
                tr("Moderne : menu dessiné aux couleurs de l'appli (police Inter, coins arrondis), qui s'ouvre en une fraction de seconde. Système : menu natif de Windows, instantané."),
                |ui| {
                    segmented(ui, &mut c.tray_menu, &[("modern".to_string(), tr("Moderne")), ("native".to_string(), tr("Système"))]);
                },
            );
        });
        self.reset_row(ui, SetCat::Look);
    }

    pub(super) fn set_privacy(&mut self, ui: &mut egui::Ui) {
        page_head(ui, icons::SHIELD, tr("Confidentialité"), tr("Évite de capturer des fenêtres sensibles par accident."));
        let c = &mut self.cfg;
        group(ui, tr("Fenêtres à ne jamais capturer"), |ui, rows| {
            row_below(
                ui,
                rows,
                tr("Mots à surveiller"),
                tr("Si le titre de la fenêtre active contient l'un de ces mots (séparés par des virgules), la capture est ignorée."),
                |ui| {
                    field(ui, &mut c.blocklist, tr("ex : keepass, bitwarden, banque"), ui.available_width());
                },
            );
        });
        self.reset_row(ui, SetCat::Privacy);
    }

    pub(super) fn set_system(&mut self, ui: &mut egui::Ui) {
        page_head(ui, icons::SYSTEM, tr("Système"), tr("Démarrage, sauvegarde des réglages et informations."));
        group(ui, tr("Langue"), |ui, rows| {
            row(
                ui,
                rows,
                tr("Langue de l'interface"),
                tr("« Automatique » suit la langue de Windows (anglais si elle n'est pas disponible). Le changement est visible tout de suite."),
                |ui| {
                    let name = |code: &str| -> String {
                        match LANGUAGES.iter().find(|l| l.0 == code) {
                            Some(("auto", _, _)) | None => tr("Automatique (langue de Windows)").to_string(),
                            Some((_, n, true)) => n.to_string(),
                            Some((_, n, false)) => format!("{n} · {}", tr("traduction non vérifiée")),
                        }
                    };
                    egui::ComboBox::new("lang", "").width(260.0).selected_text(name(&self.cfg.language)).show_ui(ui, |ui| {
                        for (code, _, _) in LANGUAGES {
                            ui.selectable_value(&mut self.cfg.language, code.to_string(), name(code));
                        }
                    });
                },
            );
        });
        group(ui, tr("Démarrage"), |ui, rows| {
            if toggle_row(ui, rows, tr("Lancer avec Windows"), tr("Shirokap démarre en silence dans la zone de notification à l'ouverture de session."), &mut self.autostart) {
                set_autostart(self.autostart);
            }
        });
        group(ui, tr("Configuration"), |ui, rows| {
            row_below(ui, rows, tr("Sauvegarder ou transférer tes réglages"), tr("Exporte tes réglages dans un fichier, ou importe-les sur un autre PC."), |ui| {
                ui.horizontal_wrapped(|ui| {
                    if pill(ui, icons::EXPORT, tr("Exporter…")).clicked() {
                        if let Some(p) = rfd::FileDialog::new().set_file_name("shirokap-config.toml").save_file() {
                            let _ = std::fs::write(p, toml::to_string_pretty(&self.cfg).unwrap_or_default());
                        }
                    }
                    if pill(ui, icons::FOLDER_OPEN, tr("Importer…")).clicked() {
                        if let Some(p) = rfd::FileDialog::new().add_filter(tr("Configuration"), &["toml"]).pick_file() {
                            if let Some(c) = std::fs::read_to_string(p).ok().and_then(|s| toml::from_str::<Config>(&s).ok()) {
                                self.cfg = c; // à enregistrer ensuite avec la pilule du bas
                            }
                        }
                    }
                    if ghost(ui, icons::FOLDER_COG, tr("Ouvrir le dossier de configuration")).clicked() {
                        if let Some(d) = config_path().parent() {
                            let _ = Command::new("explorer").arg(d).spawn();
                        }
                    }
                });
            });
        });
        group(ui, tr("Dépannage"), |ui, rows| {
            row_below(
                ui,
                rows,
                tr("Journal"),
                tr("En cas de problème (plantage, capture qui échoue, raccourci indisponible), le journal contient les détails. Joins-le quand tu signales un bug."),
                |ui| {
                    ui.horizontal_wrapped(|ui| {
                        if pill(ui, icons::BUG, tr("Ouvrir le journal")).clicked() {
                            let _ = Command::new("notepad").arg(log_path()).spawn();
                        }
                        if ghost(ui, icons::FOLDER_OPEN, tr("Ouvrir le dossier des journaux")).clicked() {
                            let _ = std::fs::create_dir_all(log_dir());
                            let _ = Command::new("explorer").arg(log_dir()).spawn();
                        }
                    });
                },
            );
        });
        group(ui, tr("À propos"), |ui, rows| {
            let title = format!("Shirokap {}", env!("CARGO_PKG_VERSION"));
            row(ui, rows, &title, tr("Police Inter © Rasmus Andersson, licence SIL Open Font License (voir assets/fonts/LICENSE-Inter.txt)."), |ui| {
                if pill(ui, icons::SPARKLES, tr("Revoir l'écran d'accueil")).clicked() {
                    self.show_welcome = true;
                }
            });
        });
        self.reset_row(ui, SetCat::System);
    }
}
