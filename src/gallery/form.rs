//! Briques des pages de réglages : en-tête de rubrique, groupes de lignes, interrupteur,
//! sélecteur en pilule, réglage numérique − champ +, champ de raccourci.
use super::*;

/// Coins extérieurs d'un groupe de lignes, et coins entre deux lignes d'un même groupe.
const R_OUT: f32 = 14.0;
const R_IN: f32 = 6.0;

/// En-tête d'une rubrique : grande tuile d'icône (couleur d'accent), titre et explication.
pub(crate) fn page_head(ui: &mut egui::Ui, icon: &str, title: &str, sub: &str) {
    let acc = accent_of(ui.visuals());
    ui.horizontal(|ui| {
        let (r, _) = ui.allocate_exact_size(vec2(44.0, 44.0), Sense::hover());
        ui.painter().rect_filled(r, 13.0, acc);
        ui.painter().text(r.center(), Align2::CENTER_CENTER, icon, FontId::proportional(21.0), on_accent(acc));
        ui.add_space(4.0);
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            ui.label(bold_text(title, 20.0));
            ui.add(egui::Label::new(egui::RichText::new(sub).size(13.5).color(ui.visuals().weak_text_color())).wrap());
        });
    });
    ui.add_space(16.0);
}

/// Fonds des lignes d'un groupe, posés une fois le groupe terminé (on sait alors laquelle est la dernière).
pub(crate) struct Rows {
    bgs: Vec<(egui::layers::ShapeIdx, Rect)>,
}

/// Groupe de réglages : un titre discret, puis des lignes collées les unes aux autres, avec des coins
/// arrondis en haut du groupe et en bas.
pub(crate) fn group(ui: &mut egui::Ui, title: &str, add: impl FnOnce(&mut egui::Ui, &mut Rows)) {
    if !title.is_empty() {
        ui.horizontal(|ui| {
            ui.add_space(4.0);
            ui.label(egui::RichText::new(title).size(14.0).family(egui::FontFamily::Name("bold".into())).color(ui.visuals().weak_text_color()));
        });
    }
    let mut rows = Rows { bgs: Vec::new() };
    ui.scope(|ui| {
        ui.spacing_mut().item_spacing.y = 4.0;
        add(ui, &mut rows);
    });
    let fill = ui.visuals().faint_bg_color;
    let n = rows.bgs.len();
    for (i, (idx, rect)) in rows.bgs.into_iter().enumerate() {
        let top = if i == 0 { R_OUT } else { R_IN };
        let bot = if i + 1 == n { R_OUT } else { R_IN };
        ui.painter().set(idx, egui::epaint::RectShape::filled(rect, Rounding { nw: top, ne: top, sw: bot, se: bot }, fill));
    }
    ui.add_space(18.0);
}

/// Titre et explication d'une ligne.
fn texts(ui: &mut egui::Ui, title: &str, desc: &str) {
    ui.spacing_mut().item_spacing.y = 3.0;
    ui.label(bold_text(title, 15.0));
    if !desc.is_empty() {
        ui.add(egui::Label::new(egui::RichText::new(desc).size(13.0).color(ui.visuals().weak_text_color())).wrap());
    }
}

/// Une ligne : titre et explication à gauche, réglage calé à droite. Si la place manque, le réglage
/// passe sous le texte. La largeur du réglage est mesurée à chaque image pour le caler au bord droit.
pub(crate) fn row<R>(ui: &mut egui::Ui, rows: &mut Rows, title: &str, desc: &str, control: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let bg = ui.painter().add(egui::Shape::Noop);
    let inner_w = (ui.available_width() - 36.0).max(80.0);
    let id = ui.id().with(("row_ctrl", title));
    let ctrl_w = ui.ctx().data(|d| d.get_temp::<f32>(id)).unwrap_or(inner_w * 0.4);
    let left_w = inner_w - ctrl_w - 24.0;
    let side = left_w >= 200.0_f32.min(inner_w * 0.45);
    let res = egui::Frame::none().inner_margin(egui::Margin::symmetric(18.0, 14.0)).show(ui, |ui| {
        ui.set_width(inner_w);
        let (inner, w) = if side {
            ui.horizontal(|ui| {
                ui.allocate_ui_with_layout(vec2(left_w, 0.0), egui::Layout::top_down(egui::Align::Min), |ui| {
                    ui.set_width(left_w);
                    texts(ui, title, desc);
                });
                ui.add_space(24.0 - ui.spacing().item_spacing.x);
                let r = ui.horizontal(control);
                (r.inner, r.response.rect.width())
            })
            .inner
        } else {
            ui.scope(|ui| texts(ui, title, desc));
            ui.add_space(6.0);
            let r = ui.horizontal(control);
            (r.inner, r.response.rect.width())
        };
        if (w - ctrl_w).abs() > 0.5 {
            ui.ctx().data_mut(|d| d.insert_temp(id, w));
            ui.ctx().request_repaint();
        }
        inner
    });
    rows.bgs.push((bg, res.response.rect));
    res.inner
}

/// Une ligne dont le réglage prend toute la largeur, sous le texte (champs, listes de vignettes…).
pub(crate) fn row_below<R>(ui: &mut egui::Ui, rows: &mut Rows, title: &str, desc: &str, content: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let bg = ui.painter().add(egui::Shape::Noop);
    let inner_w = (ui.available_width() - 36.0).max(80.0);
    let res = egui::Frame::none().inner_margin(egui::Margin::symmetric(18.0, 14.0)).show(ui, |ui| {
        ui.set_width(inner_w);
        ui.scope(|ui| texts(ui, title, desc));
        ui.add_space(4.0);
        content(ui)
    });
    rows.bgs.push((bg, res.response.rect));
    res.inner
}

/// Ligne d'information : une icône et un texte (aide des raccourcis, par exemple).
pub(crate) fn row_info(ui: &mut egui::Ui, rows: &mut Rows, icon: &str, text: &str) {
    let bg = ui.painter().add(egui::Shape::Noop);
    let inner_w = (ui.available_width() - 32.0).max(80.0);
    let res = egui::Frame::none().inner_margin(egui::Margin::symmetric(16.0, 11.0)).show(ui, |ui| {
        ui.set_width(inner_w);
        ui.horizontal(|ui| {
            let acc = accent_of(ui.visuals());
            ui.label(egui::RichText::new(icon).size(16.0).color(acc));
            ui.add_space(4.0);
            ui.add(egui::Label::new(egui::RichText::new(text).size(13.5)).wrap());
        });
    });
    rows.bgs.push((bg, res.response.rect));
}

/// Interrupteur : une piste qui se remplit de la couleur d'accent, avec une pastille qui glisse.
pub(crate) fn toggle(ui: &mut egui::Ui, on: &mut bool) -> egui::Response {
    let (rect, mut resp) = ui.allocate_exact_size(vec2(48.0, 32.0), Sense::click());
    if resp.clicked() {
        *on = !*on;
        resp.mark_changed();
    }
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let value = *on;
    resp.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, true, value, ""));
    let k = ui.ctx().animate_bool_with_time_and_easing(resp.id, value, 0.22, egui::emath::easing::cubic_out);
    let hk = ui.ctx().animate_bool_with_time(resp.id.with("hov"), resp.hovered(), super::anim::HOVER);
    let v = ui.visuals();
    let acc = accent_of(v);
    let track = Rect::from_center_size(rect.center(), vec2(44.0, 26.0));
    let p = ui.painter();
    p.rect_filled(track, 13.0, v.extreme_bg_color.lerp_to_gamma(acc, k));
    if k < 0.99 {
        let st = v.widgets.inactive.bg_stroke.color.lerp_to_gamma(v.widgets.hovered.bg_stroke.color, hk);
        p.rect_stroke(track, 13.0, Stroke::new(1.0_f32, st.gamma_multiply(1.0 - k)));
    }
    // pastille : blanche sur la couleur d'accent (ou de la teinte des boutons si l'accent est très clair)
    let lum = 0.299 * acc.r() as f32 + 0.587 * acc.g() as f32 + 0.114 * acc.b() as f32;
    let knob_on = if lum > 190.0 { v.widgets.inactive.bg_fill } else { Color32::WHITE };
    let knob = v.weak_text_color().lerp_to_gamma(knob_on, k);
    let x = egui::lerp((track.left() + 13.0)..=(track.right() - 13.0), k);
    let c = pos2(x, track.center().y);
    p.circle_filled(c + vec2(0.0, 1.0), 10.0, Color32::from_black_alpha(40));
    p.circle_filled(c, 10.0 + hk, knob);
    resp
}

/// Ligne avec un interrupteur. Renvoie vrai si on vient de le basculer.
pub(crate) fn toggle_row(ui: &mut egui::Ui, rows: &mut Rows, title: &str, desc: &str, on: &mut bool) -> bool {
    row(ui, rows, title, desc, |ui| toggle(ui, on).changed())
}

/// Sélecteur en pilule : la pastille de la couleur d'accent glisse sous l'option choisie.
pub(crate) fn segmented<T: PartialEq + Clone>(ui: &mut egui::Ui, value: &mut T, options: &[(T, &str)]) -> bool {
    let font = FontId::new(13.5, egui::FontFamily::Name("bold".into()));
    let h = 32.0;
    let galleys: Vec<_> = options.iter().map(|(_, t)| ui.painter().layout_no_wrap(t.to_string(), font.clone(), Color32::PLACEHOLDER)).collect();
    let widths: Vec<f32> = galleys.iter().map(|g| g.size().x + 30.0).collect();
    let total = widths.iter().sum::<f32>() + 2.0 * (options.len().saturating_sub(1)) as f32 + 8.0;
    let (rect, resp) = ui.allocate_exact_size(vec2(total, h + 8.0), Sense::hover());
    let id = resp.id;
    let v = ui.visuals().clone();
    let acc = accent_of(&v);
    let mut changed = false;
    let mut x = rect.min.x + 4.0;
    let mut cells = Vec::with_capacity(options.len());
    for (i, (opt, _)) in options.iter().enumerate() {
        let r = Rect::from_min_size(pos2(x, rect.min.y + 4.0), vec2(widths[i], h));
        let cell = ui.interact(r, id.with(i), Sense::click());
        if cell.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if cell.clicked() && *value != *opt {
            *value = opt.clone();
            changed = true;
        }
        let hk = ui.ctx().animate_bool_with_time(cell.id.with("hov"), cell.hovered(), super::anim::HOVER);
        cells.push((r, hk));
        x += widths[i] + 2.0;
    }
    let p = ui.painter();
    p.rect_filled(rect, (h + 8.0) / 2.0, v.extreme_bg_color);
    let sel = options.iter().position(|(o, _)| *o == *value);
    for (i, (r, hk)) in cells.iter().enumerate() {
        if Some(i) != sel && *hk > 0.01 {
            p.rect_filled(*r, h / 2.0, v.widgets.hovered.weak_bg_fill.gamma_multiply(0.7 * hk));
        }
    }
    if let Some(s) = sel {
        // la pastille glisse d'une option à l'autre (position relative au sélecteur, pour ne pas glisser au défilement)
        let ctx = ui.ctx();
        let dx = ctx.animate_value_with_time(id.with("x"), cells[s].0.min.x - rect.min.x, 0.25);
        let w = ctx.animate_value_with_time(id.with("w"), cells[s].0.width(), 0.25);
        p.rect_filled(Rect::from_min_size(pos2(rect.min.x + dx, rect.min.y + 4.0), vec2(w, h)), h / 2.0, acc);
    }
    for (i, g) in galleys.into_iter().enumerate() {
        let (r, hk) = cells[i];
        let color = if Some(i) == sel { on_accent(acc) } else { v.weak_text_color().lerp_to_gamma(v.text_color(), hk) };
        p.galley(r.center() - g.size() / 2.0, g, color);
    }
    changed
}

/// Bouton rond avec une icône (− et + des réglages numériques).
fn round_button(ui: &mut egui::Ui, icon: &str, enabled: bool) -> egui::Response {
    ui.add_enabled(enabled, egui::Button::new(egui::RichText::new(icon).size(15.0)).rounding(19.0).min_size(vec2(38.0, 38.0)))
}

/// Réglage numérique : − [ valeur unité ] +. On peut aussi taper la valeur ; Maj + clic avance de 10 pas.
pub(crate) fn stepper(ui: &mut egui::Ui, salt: &str, value: &mut u32, range: std::ops::RangeInclusive<u32>, step: u32, unit: &str) -> bool {
    let (lo, hi) = (*range.start(), *range.end());
    let mut changed = false;
    let id = ui.id().with(("stepper", salt));
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        let big = if ui.input(|i| i.modifiers.shift) { 10 } else { 1 };
        if round_button(ui, icons::MINUS, *value > lo).clicked() {
            *value = value.saturating_sub(step * big).max(lo);
            changed = true;
        }
        let v = ui.visuals().clone();
        let mut buf = ui.data(|d| d.get_temp::<String>(id)).unwrap_or_else(|| value.to_string());
        let mut focused = false;
        egui::Frame::none()
            .fill(v.extreme_bg_color)
            .stroke(v.widgets.inactive.bg_stroke)
            .rounding(12.0)
            .inner_margin(egui::Margin { left: 6.0, right: 12.0, top: 0.0, bottom: 0.0 })
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    let w = ui.painter().layout_no_wrap(hi.to_string(), FontId::new(15.0, egui::FontFamily::Name("bold".into())), Color32::WHITE).size().x + 14.0;
                    let te = egui::TextEdit::singleline(&mut buf)
                        .frame(false)
                        .desired_width(w)
                        .margin(vec2(4.0, 10.0))
                        .font(FontId::new(15.0, egui::FontFamily::Name("bold".into())));
                    let r = ui.add(te);
                    if r.changed() {
                        buf.retain(|c| c.is_ascii_digit());
                        let n: u32 = buf.parse().unwrap_or(lo);
                        let n = n.clamp(lo, hi);
                        if n != *value {
                            *value = n;
                            changed = true;
                        }
                    }
                    focused = r.has_focus();
                    ui.label(egui::RichText::new(unit).size(13.5).color(v.weak_text_color()));
                });
            });
        if !focused {
            buf = value.to_string(); // hors saisie, le champ montre la valeur réelle (bornée)
        }
        ui.data_mut(|d| d.insert_temp(id, buf));
        if round_button(ui, icons::PLUS, *value < hi).clicked() {
            *value = value.saturating_add(step * big).min(hi);
            changed = true;
        }
    });
    changed
}

/// Champ de texte aux proportions des autres réglages (hauteur 38, coins arrondis).
pub(crate) fn field(ui: &mut egui::Ui, text: &mut String, hint: &str, width: f32) -> egui::Response {
    ui.add(egui::TextEdit::singleline(text).desired_width(width).margin(vec2(12.0, 10.0)).hint_text(hint))
}

/// Bouton secondaire en pilule (teinte des boutons), avec une icône.
pub(crate) fn pill(ui: &mut egui::Ui, icon: &str, label: &str) -> egui::Response {
    let text = if icon.is_empty() { label.to_string() } else { ic(icon, label) };
    ui.add(egui::Button::new(text).rounding(18.0).min_size(vec2(0.0, 36.0)).stroke(Stroke::NONE))
}

/// Bouton discret : transparent, il ne prend une teinte qu'au survol.
pub(crate) fn ghost(ui: &mut egui::Ui, icon: &str, label: &str) -> egui::Response {
    let text = if icon.is_empty() { label.to_string() } else { ic(icon, label) };
    ui.add(egui::Button::new(text).fill(Color32::TRANSPARENT).stroke(Stroke::NONE).rounding(18.0).min_size(vec2(0.0, 36.0)))
}

/// Remet un raccourci tapé à la main sous la forme comprise par Windows : noms français, allemands
/// ou espagnols des touches (Maj, Strg, Espace…), espaces autour des « + », minuscules.
pub(crate) fn normalize_hotkey(s: &str) -> String {
    let parts: Vec<String> = s
        .split('+')
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .map(|p| {
            let l = p.to_lowercase();
            let canon = match l.as_str() {
                "maj" | "shift" | "umschalt" | "mayús" | "mayus" | "shift ⇧" => "Shift",
                "ctrl" | "control" | "strg" | "contrôle" | "controle" => "Ctrl",
                "alt" => "Alt",
                "win" | "windows" | "super" | "meta" => "Super",
                "espace" | "space" | "leertaste" | "espacio" => "Space",
                "suppr" | "del" | "delete" | "entf" | "supr" => "Delete",
                "inser" | "ins" | "insert" | "einfg" => "Insert",
                "impr. écran" | "impr écran" | "imprécran" | "printscreen" | "print" | "druck" | "impr" => "PrintScreen",
                "pause" => "Pause",
                "origine" | "home" | "pos1" | "inicio" => "Home",
                "fin" | "end" | "ende" => "End",
                "pg. préc." | "pgprec" | "pageup" | "bild↑" | "repág" => "PageUp",
                "pg. suiv." | "pgsuiv" | "pagedown" | "bild↓" | "avpág" => "PageDown",
                "haut" | "up" | "↑" => "Up",
                "bas" | "down" | "↓" => "Down",
                "gauche" | "left" | "←" => "Left",
                "droite" | "right" | "→" => "Right",
                "tab" | "tabulation" => "Tab",
                _ => "",
            };
            if !canon.is_empty() {
                canon.to_string()
            } else if l.len() > 1 && l.starts_with('f') && l[1..].parse::<u8>().is_ok() {
                l.to_uppercase()
            } else if l.chars().count() == 1 {
                l.to_uppercase()
            } else {
                p.to_string()
            }
        })
        .collect();
    parts.join("+")
}

/// Ligne d'un raccourci : nom, champ (on peut y écrire la combinaison), « Enregistrer » pour l'appuyer
/// au clavier, « Désactiver ». Dessous : erreur ou avertissement éventuel.
pub(crate) fn hotkey_row(ui: &mut egui::Ui, rows: &mut Rows, label: &str, val: &mut String, rec: &mut Option<usize>, idx: usize) {
    let bg = ui.painter().add(egui::Shape::Noop);
    let inner_w = (ui.available_width() - 30.0).max(80.0);
    let recording = *rec == Some(idx);
    let res = egui::Frame::none().inner_margin(egui::Margin { left: 18.0, right: 12.0, top: 10.0, bottom: 10.0 }).show(ui, |ui| {
        ui.set_width(inner_w);
        let v = ui.visuals().clone();
        let acc = accent_of(&v);
        ui.horizontal_wrapped(|ui| {
            let lw = (inner_w * 0.32).clamp(140.0, 210.0);
            ui.allocate_ui_with_layout(vec2(lw, 38.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.set_min_size(vec2(lw, 38.0));
                ui.add(egui::Label::new(bold_text(label, 15.0)).truncate());
            });
            if recording {
                // le champ attend les touches : contour de la couleur d'accent
                egui::Frame::none()
                    .fill(v.extreme_bg_color)
                    .stroke(Stroke::new(1.5_f32, acc))
                    .rounding(12.0)
                    .inner_margin(egui::Margin::symmetric(12.0, 9.0))
                    .show(ui, |ui| {
                        ui.set_width(196.0);
                        ui.horizontal(|ui| {
                            let (d, _) = ui.allocate_exact_size(vec2(8.0, 8.0), Sense::hover());
                            ui.painter().circle_filled(d.center(), 3.5, acc);
                            ui.label(egui::RichText::new(tr("Appuie sur les touches…")).color(acc.lerp_to_gamma(v.text_color(), 0.35)));
                        });
                    });
                if pill(ui, "", tr("Annuler")).clicked() {
                    *rec = None;
                }
            } else {
                let r = field(ui, val, tr("désactivé"), 196.0);
                if r.lost_focus() {
                    *val = normalize_hotkey(val);
                }
                if pill(ui, icons::KEYBOARD, tr("Enregistrer|raccourci")).on_hover_text(tr("Clique, puis appuie sur la touche ou la combinaison")).clicked() {
                    *rec = Some(idx);
                }
                if ui.add_enabled_ui(!val.is_empty(), |ui| ghost(ui, icons::CLOSE, tr("Désactiver"))).inner.clicked() {
                    val.clear();
                }
            }
        });
        // message sous la ligne
        let line = |ui: &mut egui::Ui, icon: &str, text: &str, color: Color32| {
            ui.horizontal_wrapped(|ui| {
                ui.add_space(4.0);
                ui.label(egui::RichText::new(icon).color(color));
                ui.label(egui::RichText::new(text).size(13.0).color(color));
            });
        };
        if recording {
            line(ui, icons::INFO, tr("Échap : annuler · Retour arrière : vider"), v.weak_text_color());
        } else if !val.is_empty() && val.parse::<global_hotkey::hotkey::HotKey>().is_err() {
            line(ui, icons::ALERT, tr("Touche non reconnue. Exemples : F7, Pause, Ctrl+Shift+S, Alt+F12."), col(248, 113, 113));
        } else if lone_typing_key(val) {
            line(ui, icons::ALERT, tr("Touche seule : elle ne pourra plus être tapée dans les autres applications."), col(251, 191, 36));
        }
    });
    rows.bgs.push((bg, res.response.rect));
}

/// Champ de recherche en pilule : loupe à gauche, croix pour effacer à droite.
pub(crate) fn search_field(ui: &mut egui::Ui, text: &mut String, hint: &str, width: f32) -> egui::Response {
    let v = ui.visuals().clone();
    let weak = v.weak_text_color();
    egui::Frame::none()
        .fill(v.extreme_bg_color)
        .stroke(v.widgets.inactive.bg_stroke)
        .rounding(19.0)
        .inner_margin(egui::Margin { left: 14.0, right: 6.0, top: 0.0, bottom: 0.0 })
        .show(ui, |ui| {
            ui.set_width(width);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                ui.label(egui::RichText::new(icons::SEARCH).size(15.0).color(weak));
                let w = ui.available_width() - if text.is_empty() { 0.0 } else { 30.0 };
                let r = ui.add(egui::TextEdit::singleline(text).frame(false).desired_width(w).margin(vec2(2.0, 10.0)).hint_text(hint));
                if !text.is_empty() {
                    let (cr, c) = ui.allocate_exact_size(vec2(26.0, 26.0), Sense::click());
                    let c = c.on_hover_text(tr("Effacer la recherche"));
                    if c.hovered() {
                        ui.painter().circle_filled(cr.center(), 13.0, v.widgets.hovered.weak_bg_fill);
                    }
                    ui.painter().text(cr.center(), Align2::CENTER_CENTER, icons::CLOSE, FontId::proportional(13.0), weak);
                    if c.clicked() {
                        text.clear();
                    }
                }
                r
            })
            .inner
        })
        .inner
}

/// Pastille de filtre : pleine de la couleur d'accent quand elle est active.
pub(crate) fn chip(ui: &mut egui::Ui, on: bool, icon: &str, label: &str) -> egui::Response {
    let acc = accent_of(ui.visuals());
    let text = if icon.is_empty() { label.to_string() } else { ic(icon, label) };
    let rich = if on {
        egui::RichText::new(text).color(on_accent(acc)).family(egui::FontFamily::Name("bold".into()))
    } else {
        egui::RichText::new(text)
    };
    let fill = if on { acc } else { ui.visuals().faint_bg_color };
    ui.add(egui::Button::new(rich).fill(fill).stroke(Stroke::NONE).rounding(17.0).min_size(vec2(0.0, 34.0)))
}

/// Pastille « Favoris » : jaune quand le filtre est actif.
pub(crate) fn fav_chip(ui: &mut egui::Ui, on: bool, label: &str) -> egui::Response {
    let gold = col(251, 191, 36);
    let text = ic(if on { "★" } else { icons::STAR }, label);
    let rich = if on {
        egui::RichText::new(text).color(Color32::from_rgb(28, 22, 8)).family(egui::FontFamily::Name("bold".into()))
    } else {
        egui::RichText::new(text)
    };
    let fill = if on { gold } else { ui.visuals().faint_bg_color };
    ui.add(egui::Button::new(rich).fill(fill).stroke(Stroke::NONE).rounding(17.0).min_size(vec2(0.0, 34.0)))
}
