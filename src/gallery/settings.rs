// Page des paramètres (par rubriques) et réinitialisations.
use super::*;

impl App {
    // ----- paramètres -----
    pub(super) fn settings_ui(&mut self, ui: &mut egui::Ui) {
        ui.label(bold_text(tr("Paramètres"), 24.0));
        ui.add_space(6.0);
        egui::SidePanel::left("set_nav")
            .exact_width(170.0)
            .resizable(false)
            .frame(egui::Frame::none())
            .show_inside(ui, |ui| {
                for (c, i, n) in [
                    (SetCat::Shortcuts, icons::KEYBOARD, tr("Raccourcis")),
                    (SetCat::Capture, icons::CAMERA, tr("Capture")),
                    (SetCat::Files, icons::FOLDER, tr("Fichiers")),
                    (SetCat::Look, icons::PALETTE, tr("Apparence")),
                    (SetCat::Privacy, icons::SHIELD, tr("Confidentialité")),
                    (SetCat::System, icons::SYSTEM, tr("Système")),
                ] {
                    nav_item(ui, &mut self.set_cat, c, i, n);
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
        let pad = egui::Margin { left: 18.0 + 28.0 * (1.0 - k), right: 0.0, top: 0.0, bottom: 0.0 };
        egui::Frame::none().inner_margin(pad).show(ui, |ui| {
            ui.multiply_opacity(k);
            egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| match self.set_cat {
                SetCat::Shortcuts => self.set_shortcuts(ui),
                SetCat::Capture => self.set_capture(ui),
                SetCat::Files => self.set_files(ui),
                SetCat::Look => self.set_look(ui),
                SetCat::Privacy => self.set_privacy(ui),
                SetCat::System => self.set_system(ui),
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
                c.mica = d.mica;
                c.tray_menu = d.tray_menu;
            }
            SetCat::Privacy => c.blocklist = d.blocklist,
            SetCat::System => c.language = d.language,
        }
        self.status = tr("Rubrique réinitialisée : pense à enregistrer").into();
    }

    pub(super) fn reset_row(&mut self, ui: &mut egui::Ui, cat: SetCat) {
        ui.add_space(2.0);
        if ui
            .button(ic(icons::RESET, tr("Rétablir les valeurs par défaut de cette rubrique")))
            .on_hover_text(tr("Remet uniquement les réglages de cette page à leur valeur d'origine. Rien n'est appliqué tant que tu n'as pas cliqué sur « Enregistrer »."))
            .clicked()
        {
            self.reset_cat(cat);
        }
        ui.add_space(14.0);
    }

    pub(super) fn set_shortcuts(&mut self, ui: &mut egui::Ui) {
        page_head(ui, tr("Raccourcis"), tr("Des combinaisons de touches qui fonctionnent partout dans Windows. Clique sur « Enregistrer », puis appuie sur ta combinaison."));
        card(ui, tr("Capture"), |ui| {
            hotkey_row(ui, tr("Capture"), &mut self.cfg.hotkey, &mut self.recording, 0);
            hotkey_row(ui, tr("Zone (sélection)"), &mut self.cfg.hotkey_region, &mut self.recording, 1);
            hotkey_row(ui, tr("Fenêtre active"), &mut self.cfg.hotkey_window, &mut self.recording, 2);
            hotkey_row(ui, tr("Dernière zone"), &mut self.cfg.hotkey_last_region, &mut self.recording, 3);
            hotkey_row(ui, tr("Zone → presse-papiers"), &mut self.cfg.hotkey_region_clip, &mut self.recording, 5);
            hotkey_row(ui, tr("Pipette (couleur)"), &mut self.cfg.hotkey_picker, &mut self.recording, 6);
        });
        card(ui, tr("Application"), |ui| {
            hotkey_row(ui, tr("Ouvrir la galerie"), &mut self.cfg.hotkey_gallery, &mut self.recording, 4);
            ui.add_space(4.0);
            ui.weak(tr("Laisse un champ vide pour désactiver le raccourci. Les touches F1 à F24, Impr. écran, Pause… peuvent s'utiliser seules ; les autres demandent Ctrl, Alt ou Maj."));
        });
        card(ui, tr("Dans la galerie"), |ui| {
            for l in [
                tr("Clic : ouvrir  ·  Ctrl+clic : sélectionner  ·  clic droit : menu  ·  glisser des images dessus : les importer"),
                tr("Ctrl+A : tout sélectionner  ·  Suppr : corbeille  ·  Échap : désélectionner  ·  Ctrl+B : menu latéral"),
                tr("Visionneuse : ← → naviguer  ·  F favori  ·  Espace diaporama  ·  F11 plein écran  ·  molette zoom"),
                tr("Capture épinglée : glisser pour déplacer  ·  molette pour la taille  ·  clic droit ou Échap pour fermer"),
                tr("Icône du tray : clic gauche = galerie  ·  clic molette = capture  ·  clic droit = menu"),
            ] {
                ui.label(l);
            }
        });
        self.reset_row(ui, SetCat::Shortcuts);
    }

    pub(super) fn set_capture(&mut self, ui: &mut egui::Ui) {
        page_head(ui, tr("Capture"), tr("Comment l'écran est capturé, et ce qui se passe juste après."));
        card(ui, tr("Source"), |ui| {
            setting(ui, tr("Écran à capturer"), tr("L'écran utilisé par le raccourci de capture."), |ui| {
                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.cfg.monitor_mode, "primary".to_string(), tr("Principal"));
                    ui.radio_value(&mut self.cfg.monitor_mode, "cursor".to_string(), tr("Sous la souris"));
                    ui.radio_value(&mut self.cfg.monitor_mode, "all".to_string(), tr("Tous les écrans"));
                });
            });
            divider(ui);
            setting(
                ui,
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
                    egui::ComboBox::new("res", "").selected_text(label(self.cfg.target_width)).show_ui(ui, |ui| {
                        for w in [0u32, 3840, 2560, 1920, 1280] {
                            ui.selectable_value(&mut self.cfg.target_width, w, label(w));
                        }
                    });
                },
            );
            divider(ui);
            setting(ui, tr("Délai avant capture"), tr("Attend quelques secondes avant de capturer : pratique pour ouvrir un menu ou changer de fenêtre."), |ui| {
                ui.add(egui::Slider::new(&mut self.cfg.delay_secs, 0..=10).suffix(" s"));
            });
        });
        card(ui, "Timelapse", |ui| {
            setting(ui, tr("Intervalle"), tr("Une capture toutes les N secondes quand le timelapse est activé depuis le menu de l'icône du tray."), |ui| {
                ui.add(egui::Slider::new(&mut self.cfg.interval_secs, 5..=600).suffix(" s"));
            });
        });
        card(ui, tr("Après la capture"), |ui| {
            setting(ui, tr("Copier dans le presse-papiers"), tr("L'image est prête à être collée (Ctrl+V) dans un message, un document…"), |ui| {
                ui.checkbox(&mut self.cfg.clipboard, tr("Activé"));
            });
            divider(ui);
            setting(ui, tr("Clignotement de l'icône"), tr("L'icône du tray devient verte un instant pour confirmer la capture."), |ui| {
                ui.checkbox(&mut self.cfg.flash, tr("Activé"));
            });
            divider(ui);
            setting(ui, tr("Bip sonore"), tr("Joue un petit son à chaque capture (assets/beep.wav)."), |ui| {
                ui.checkbox(&mut self.cfg.sound, tr("Activé"));
            });
        });
        self.reset_row(ui, SetCat::Capture);
    }

    pub(super) fn set_files(&mut self, ui: &mut egui::Ui) {
        page_head(ui, tr("Fichiers"), tr("Où et comment les captures sont enregistrées."));
        card(ui, tr("Enregistrement"), |ui| {
            setting(ui, tr("Dossier des captures"), tr("Le dossier est créé automatiquement s'il n'existe pas."), |ui| {
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut self.cfg.save_dir).desired_width(380.0));
                    if ui.button(tr("Parcourir…")).clicked() {
                        if let Some(p) = rfd::FileDialog::new().pick_folder() {
                            self.cfg.save_dir = p.display().to_string();
                        }
                    }
                });
            });
            divider(ui);
            setting(ui, tr("Nom des fichiers"), tr("Variables : {window} nom de la fenêtre · {date} · {time} · {res} résolution · {n} numéro."), |ui| {
                ui.add(egui::TextEdit::singleline(&mut self.cfg.name_template).desired_width(300.0));
            });
            divider(ui);
            setting(ui, tr("Format"), tr("PNG garde une qualité parfaite ; JPEG est plus léger."), |ui| {
                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.cfg.format, "png".to_string(), "PNG");
                    ui.radio_value(&mut self.cfg.format, "jpg".to_string(), "JPEG");
                });
                if self.cfg.format == "jpg" {
                    ui.add(egui::Slider::new(&mut self.cfg.jpg_quality, 50..=100).text(tr("qualité")));
                } else {
                    ui.horizontal(|ui| {
                        ui.label(tr("Compression :"));
                        ui.radio_value(&mut self.cfg.png_level, 0u8, tr("Rapide"));
                        ui.radio_value(&mut self.cfg.png_level, 1u8, tr("Normale"));
                        ui.radio_value(&mut self.cfg.png_level, 2u8, tr("Maximale"));
                    });
                }
            });
            divider(ui);
            setting(ui, tr("Un sous-dossier par fenêtre ou jeu"), tr("Range chaque capture dans un dossier au nom de la fenêtre active."), |ui| {
                ui.checkbox(&mut self.cfg.folder_per_window, tr("Activé"));
            });
        });
        card(ui, tr("Rangement automatique"), |ui| {
            setting(
                ui,
                tr("Vider les vieilles captures"),
                tr("Met à la corbeille les captures plus anciennes que ce délai. 0 = désactivé. Les favoris sont toujours conservés, et la corbeille Windows permet de tout récupérer."),
                |ui| {
                    ui.add(egui::Slider::new(&mut self.cfg.cleanup_days, 0..=365).suffix(tr(" jours")));
                },
            );
        });
        card(ui, tr("Éditeur externe"), |ui| {
            setting(ui, tr("Programme d'édition"), tr("Utilisé par le bouton « Modifier dans l'éditeur externe » (Paint par défaut)."), |ui| {
                ui.add(egui::TextEdit::singleline(&mut self.cfg.editor).desired_width(300.0).hint_text("mspaint"));
            });
        });
        self.reset_row(ui, SetCat::Files);
    }

    pub(super) fn set_look(&mut self, ui: &mut egui::Ui) {
        page_head(ui, tr("Apparence"), tr("Le thème et la couleur de l'interface. Les changements sont visibles tout de suite."));
        // les cartes de thème puis les pastilles d'accent apparaissent en cascade à l'arrivée sur la page
        let since = self.tab_at.elapsed().min(self.cat_at.elapsed()).as_secs_f32();
        let appear = |n: usize| ease_out(((since - 0.1 - n as f32 * 0.035) / 0.45).clamp(0.0, 1.0));
        if appear(THEMES.len() + ACCENTS.len() + 1) < 1.0 {
            ui.ctx().request_repaint();
        }
        card(ui, tr("Thème"), |ui| {
            setting(ui, tr("Thème"), tr("Choisis l'ambiance de l'interface : le changement est immédiat."), |ui| {
                let fixed = self.cfg.accent;
                let auto = self.cfg.accent_auto;
                ui.horizontal_wrapped(|ui| {
                    for (n, (key, name)) in THEMES.iter().enumerate() {
                        let pal = palette(key);
                        // en mode « Auto », chaque vignette montre la couleur d'accent de son propre thème
                        let a = if auto { theme_accent(key) } else { fixed };
                        if theme_card(ui, self.cfg.theme == *key, tr(name), &pal, col(a[0], a[1], a[2]), appear(n)).clicked() {
                            self.cfg.theme = key.to_string();
                        }
                    }
                });
            });
            divider(ui);
            setting(ui, tr("Couleur d'accent"), tr("Boutons principaux, sélection, barres et liseré du menu. Choisis une couleur rapide ou une couleur libre. « Auto » prend la couleur propre à chaque thème."), |ui| {
                ui.horizontal(|ui| {
                    let ctx = ui.ctx().clone();
                    let now = ctx.input(|i| i.time);
                    let rip_id = egui::Id::new("accent_ripple");
                    let ripple = ctx.data(|d| d.get_temp::<(usize, f64)>(rip_id));
                    // bouton « Auto » : la couleur suit le thème. Pastille aux couleurs du thème actuel + libellé,
                    // dans une capsule pour le distinguer des couleurs fixes
                    {
                        let ta = theme_accent(&self.cfg.theme);
                        let tcol = col(ta[0], ta[1], ta[2]);
                        let (r, resp) = ui.allocate_exact_size(vec2(84.0, 28.0), Sense::click());
                        let resp = resp.on_hover_text(tr("Couleur du thème : l'accent suit le thème choisi"));
                        if resp.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        let on = self.cfg.accent_auto;
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
                        let c = pos2(r.min.x + 15.0, r.center().y);
                        let spin = if resp.hovered() { ctx.input(|i| i.time) as f32 * 1.2 } else { 0.0 }; // tourne au survol
                        let segs = [theme_accent("ocean"), theme_accent("neon"), theme_accent("ruby"), theme_accent("mocha"), theme_accent("forest"), theme_accent("midnight")];
                        for (j, sc) in segs.iter().enumerate() {
                            let a0 = j as f32 / segs.len() as f32 * std::f32::consts::TAU;
                            let a1 = (j + 1) as f32 / segs.len() as f32 * std::f32::consts::TAU;
                            let pts: Vec<Pos2> = std::iter::once(c)
                                .chain((0..=6).map(|t| {
                                    let a = a0 + (a1 - a0) * t as f32 / 6.0 + spin;
                                    c + vec2(a.cos(), a.sin()) * 9.0
                                }))
                                .collect();
                            p.add(egui::Shape::convex_polygon(pts, col(sc[0], sc[1], sc[2]), Stroke::NONE));
                        }
                        p.circle_filled(c, 5.0, tcol);
                        p.circle_stroke(c, 5.0, Stroke::new(1.5_f32, v.widgets.inactive.bg_fill));
                        p.text(pos2(c.x + 14.0, r.center().y), Align2::LEFT_CENTER, tr("Auto|accent"), FontId::new(14.0, egui::FontFamily::Name("bold".into())), txt);
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
                            self.cfg.accent_auto = true;
                            ctx.data_mut(|d| d.insert_temp(rip_id, (usize::MAX, now)));
                        }
                        ui.add_space(10.0);
                    }
                    for (n, c) in ACCENTS.iter().enumerate() {
                        let (r, resp) = ui.allocate_exact_size(vec2(28.0, 28.0), Sense::click());
                        if resp.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        let chosen = !self.cfg.accent_auto && self.cfg.accent == *c;
                        // survol : la pastille grossit ; choix : l'anneau se referme autour en rebondissant
                        let hk = ctx.animate_bool_with_time_and_easing(resp.id.with("hov"), resp.hovered(), anim::HOVER, egui::emath::easing::back_out);
                        let sk = ctx.animate_bool_with_time_and_easing(resp.id.with("sel"), chosen, 0.4, egui::emath::easing::back_out);
                        let k = appear(THEMES.len() + 1 + n);
                        let ring = ui.visuals().text_color();
                        let color = col(c[0], c[1], c[2]);
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
                            self.cfg.accent = *c;
                            self.cfg.accent_auto = false;
                            ctx.data_mut(|d| d.insert_temp(rip_id, (n, now)));
                        }
                    }
                    ui.add_space(8.0);
                    // couleur libre : elle montre la couleur affichée ; la modifier repasse en couleur fixe
                    let mut free = self.cfg.accent_rgb();
                    color_button(ui, "accent", &mut free);
                    if free != self.cfg.accent_rgb() {
                        self.cfg.accent = free;
                        self.cfg.accent_auto = false;
                    }
                });
            });
        });
        card(ui, &ic(icons::SPARKLES, tr("Matière")), |ui| {
            setting(
                ui,
                tr("Effet Mica (Windows 11)"),
                tr("Fond légèrement translucide qui laisse deviner ton fond d'écran, comme les applis récentes de Windows 11. Sans effet sous Windows 10."),
                |ui| {
                    ui.checkbox(&mut self.cfg.mica, tr("Activé"));
                },
            );
        });
        card(ui, tr("Menu de l'icône du tray"), |ui| {
            setting(
                ui,
                tr("Style du menu"),
                tr("Moderne : menu dessiné aux couleurs de l'appli (police Inter, coins arrondis), qui s'ouvre en une fraction de seconde. Système : menu natif de Windows, instantané."),
                |ui| {
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut self.cfg.tray_menu, "modern".to_string(), tr("Moderne"));
                        ui.radio_value(&mut self.cfg.tray_menu, "native".to_string(), tr("Système"));
                    });
                },
            );
        });
        self.reset_row(ui, SetCat::Look);
    }

    pub(super) fn set_privacy(&mut self, ui: &mut egui::Ui) {
        page_head(ui, tr("Confidentialité"), tr("Évite de capturer des fenêtres sensibles par accident."));
        card(ui, tr("Fenêtres à ne jamais capturer"), |ui| {
            setting(
                ui,
                tr("Mots à surveiller"),
                tr("Si le titre de la fenêtre active contient l'un de ces mots (séparés par des virgules), la capture est ignorée."),
                |ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.cfg.blocklist)
                            .desired_width(f32::INFINITY)
                            .hint_text(tr("ex : keepass, bitwarden, banque")),
                    );
                },
            );
        });
        self.reset_row(ui, SetCat::Privacy);
    }

    pub(super) fn set_system(&mut self, ui: &mut egui::Ui) {
        page_head(ui, tr("Système"), tr("Démarrage, sauvegarde des réglages et informations."));
        card(ui, tr("Langue"), |ui| {
            setting(
                ui,
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
                    egui::ComboBox::new("lang", "").width(300.0).selected_text(name(&self.cfg.language)).show_ui(ui, |ui| {
                        for (code, _, _) in LANGUAGES {
                            ui.selectable_value(&mut self.cfg.language, code.to_string(), name(code));
                        }
                    });
                },
            );
        });
        card(ui, tr("Démarrage"), |ui| {
            setting(ui, tr("Lancer avec Windows"), tr("Shirokap démarre en silence dans la zone de notification à l'ouverture de session."), |ui| {
                if ui.checkbox(&mut self.autostart, tr("Activé")).changed() {
                    set_autostart(self.autostart);
                }
            });
        });
        card(ui, tr("Configuration"), |ui| {
            setting(ui, tr("Sauvegarder ou transférer tes réglages"), tr("Exporte tes réglages dans un fichier, ou importe-les sur un autre PC."), |ui| {
                ui.horizontal(|ui| {
                    if ui.button(tr("Exporter…")).clicked() {
                        if let Some(p) = rfd::FileDialog::new().set_file_name("shirokap-config.toml").save_file() {
                            let _ = std::fs::write(p, toml::to_string_pretty(&self.cfg).unwrap_or_default());
                        }
                    }
                    if ui.button(tr("Importer…")).clicked() {
                        if let Some(p) = rfd::FileDialog::new().add_filter(tr("Configuration"), &["toml"]).pick_file() {
                            if let Some(c) = std::fs::read_to_string(p).ok().and_then(|s| toml::from_str::<Config>(&s).ok()) {
                                self.cfg = c; // à enregistrer ensuite avec le bouton du bas
                            }
                        }
                    }
                    if ui.button(tr("Ouvrir le dossier de configuration")).clicked() {
                        if let Some(d) = config_path().parent() {
                            let _ = Command::new("explorer").arg(d).spawn();
                        }
                    }
                });
            });
        });
        card(ui, tr("Dépannage"), |ui| {
            setting(
                ui,
                tr("Journal"),
                tr("En cas de problème (plantage, capture qui échoue, raccourci indisponible), le journal contient les détails. Joins-le quand tu signales un bug."),
                |ui| {
                    ui.horizontal(|ui| {
                        if ui.button(tr("Ouvrir le journal")).clicked() {
                            let _ = Command::new("notepad").arg(log_path()).spawn();
                        }
                        if ui.button(tr("Ouvrir le dossier des journaux")).clicked() {
                            let _ = std::fs::create_dir_all(log_dir());
                            let _ = Command::new("explorer").arg(log_dir()).spawn();
                        }
                    });
                },
            );
        });
        card(ui, tr("À propos"), |ui| {
            ui.label(format!("Shirokap {}", env!("CARGO_PKG_VERSION")));
            ui.weak(tr("Police Inter © Rasmus Andersson, licence SIL Open Font License (voir assets/fonts/LICENSE-Inter.txt)."));
            ui.add_space(4.0);
            if ui.button(tr("Revoir l'écran d'accueil")).clicked() {
                self.show_welcome = true;
            }
        });
        self.reset_row(ui, SetCat::System);
    }
}
