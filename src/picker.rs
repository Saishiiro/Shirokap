use crate::i18n::*;
use eframe::egui::{self, pos2, vec2, Color32, Key, Rect, Sense, Stroke};

#[derive(Clone, Default)]
struct PickState {
    open: bool,
    hsv: [f32; 3],
    hex: String,
}

/// Couleur de texte lisible sur une couleur d'accent : noir sur les couleurs claires, blanc sinon.
pub fn on_accent(c: Color32) -> Color32 {
    let lum = 0.299 * c.r() as f32 + 0.587 * c.g() as f32 + 0.114 * c.b() as f32;
    if lum > 160.0 {
        Color32::BLACK
    } else {
        Color32::WHITE
    }
}

fn rgb_color(c: [u8; 3]) -> Color32 {
    Color32::from_rgb(c[0], c[1], c[2])
}

fn hex_of(c: [u8; 3]) -> String {
    format!("#{:02X}{:02X}{:02X}", c[0], c[1], c[2])
}

fn parse_hex(s: &str) -> Option<[u8; 3]> {
    let t = s.trim().trim_start_matches('#');
    if t.len() != 6 {
        return None;
    }
    let v = u32::from_str_radix(t, 16).ok()?;
    Some([(v >> 16) as u8, (v >> 8) as u8, v as u8])
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [u8; 3] {
    let h = ((h.fract() + 1.0).fract()) * 6.0;
    let (i, f) = (h.floor() as i32, h - h.floor());
    let (p, q, t) = (v * (1.0 - s), v * (1.0 - f * s), v * (1.0 - (1.0 - f) * s));
    let (r, g, b) = match i % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };
    [(r * 255.0).round() as u8, (g * 255.0).round() as u8, (b * 255.0).round() as u8]
}

fn rgb_to_hsv(c: [u8; 3]) -> [f32; 3] {
    let (r, g, b) = (c[0] as f32 / 255.0, c[1] as f32 / 255.0, c[2] as f32 / 255.0);
    let (mx, mn) = (r.max(g).max(b), r.min(g).min(b));
    let d = mx - mn;
    let h = if d == 0.0 {
        0.0
    } else if mx == r {
        ((g - b) / d).rem_euclid(6.0) / 6.0
    } else if mx == g {
        ((b - r) / d + 2.0) / 6.0
    } else {
        ((r - g) / d + 4.0) / 6.0
    };
    [h, if mx == 0.0 { 0.0 } else { d / mx }, mx]
}

/// Rectangle dégradé (couleurs des quatre coins).
fn grad(p: &egui::Painter, rect: Rect, tl: Color32, tr: Color32, br: Color32, bl: Color32) {
    let mut m = egui::Mesh::default();
    m.colored_vertex(rect.left_top(), tl);
    m.colored_vertex(rect.right_top(), tr);
    m.colored_vertex(rect.right_bottom(), br);
    m.colored_vertex(rect.left_bottom(), bl);
    m.add_triangle(0, 1, 2);
    m.add_triangle(0, 2, 3);
    p.add(egui::Shape::mesh(m));
}

/// Sélecteur de couleur entièrement en français : une pastille qui ouvre un petit panneau
/// (carré saturation / luminosité, barre de teinte, code hexadécimal, valeurs R V B, bouton Copier).
pub fn color_button(ui: &mut egui::Ui, id: &str, rgb: &mut [u8; 3]) -> bool {
    let sid = egui::Id::new(("couleur", id));
    let mut st: PickState = ui.ctx().data_mut(|d| d.get_temp(sid)).unwrap_or_default();
    let mut changed = false;

    let (rect, resp) = ui.allocate_exact_size(vec2(46.0, 28.0), Sense::click());
    let resp = resp.on_hover_text(tr("Choisir une couleur"));
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let line = ui.visuals().widgets.inactive.bg_stroke.color;
    ui.painter().rect_filled(rect, 8.0, rgb_color(*rgb));
    ui.painter().rect_stroke(rect, 8.0, Stroke::new(1.5_f32, line));
    if resp.clicked() {
        st.open = !st.open;
        if st.open {
            st.hsv = rgb_to_hsv(*rgb);
            st.hex = hex_of(*rgb);
        }
    }

    if st.open {
        let area = egui::Area::new(sid.with("panneau"))
            .order(egui::Order::Foreground)
            .fixed_pos(rect.left_bottom() + vec2(0.0, 6.0))
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style()).inner_margin(egui::Margin::same(12.0)).show(ui, |ui| {
                    ui.spacing_mut().item_spacing = vec2(8.0, 8.0);

                    // carré saturation (→) / luminosité (↑)
                    let (sv_rect, sv) = ui.allocate_exact_size(vec2(240.0, 160.0), Sense::click_and_drag());
                    let hue = rgb_color(hsv_to_rgb(st.hsv[0], 1.0, 1.0));
                    grad(ui.painter(), sv_rect, Color32::WHITE, hue, hue, Color32::WHITE);
                    grad(ui.painter(), sv_rect, Color32::TRANSPARENT, Color32::TRANSPARENT, Color32::BLACK, Color32::BLACK);
                    if sv.dragged() || sv.clicked() {
                        if let Some(pos) = sv.interact_pointer_pos() {
                            st.hsv[1] = ((pos.x - sv_rect.min.x) / sv_rect.width()).clamp(0.0, 1.0);
                            st.hsv[2] = 1.0 - ((pos.y - sv_rect.min.y) / sv_rect.height()).clamp(0.0, 1.0);
                            *rgb = hsv_to_rgb(st.hsv[0], st.hsv[1], st.hsv[2]);
                            st.hex = hex_of(*rgb);
                            changed = true;
                        }
                    }
                    let mp = pos2(
                        sv_rect.min.x + st.hsv[1] * sv_rect.width(),
                        sv_rect.min.y + (1.0 - st.hsv[2]) * sv_rect.height(),
                    );
                    ui.painter().circle_stroke(mp, 7.0, Stroke::new(1.0_f32, Color32::BLACK));
                    ui.painter().circle_stroke(mp, 6.0, Stroke::new(2.0_f32, Color32::WHITE));

                    // barre de teinte
                    let (hr, hs) = ui.allocate_exact_size(vec2(240.0, 16.0), Sense::click_and_drag());
                    for k in 0..6 {
                        let x0 = hr.min.x + hr.width() * k as f32 / 6.0;
                        let x1 = hr.min.x + hr.width() * (k + 1) as f32 / 6.0;
                        let c0 = rgb_color(hsv_to_rgb(k as f32 / 6.0, 1.0, 1.0));
                        let c1 = rgb_color(hsv_to_rgb((k + 1) as f32 / 6.0, 1.0, 1.0));
                        grad(ui.painter(), Rect::from_min_max(pos2(x0, hr.min.y), pos2(x1, hr.max.y)), c0, c1, c1, c0);
                    }
                    if hs.dragged() || hs.clicked() {
                        if let Some(pos) = hs.interact_pointer_pos() {
                            st.hsv[0] = ((pos.x - hr.min.x) / hr.width()).clamp(0.0, 0.999);
                            *rgb = hsv_to_rgb(st.hsv[0], st.hsv[1], st.hsv[2]);
                            st.hex = hex_of(*rgb);
                            changed = true;
                        }
                    }
                    let hx = hr.min.x + st.hsv[0] * hr.width();
                    let handle = Rect::from_center_size(pos2(hx, hr.center().y), vec2(6.0, 20.0));
                    ui.painter().rect_stroke(handle, 3.0, Stroke::new(3.0_f32, Color32::BLACK));
                    ui.painter().rect_stroke(handle, 3.0, Stroke::new(1.6_f32, Color32::WHITE));

                    // code hexadécimal + copie
                    ui.horizontal(|ui| {
                        ui.label(tr("Code"));
                        let r = ui.add(egui::TextEdit::singleline(&mut st.hex).desired_width(86.0));
                        if r.changed() {
                            if let Some(c) = parse_hex(&st.hex) {
                                *rgb = c;
                                st.hsv = rgb_to_hsv(c);
                                changed = true;
                            }
                        }
                        if ui.button(tr("Copier")).on_hover_text(tr("Copier le code de la couleur")).clicked() {
                            ui.ctx().copy_text(hex_of(*rgb));
                        }
                    });
                    // valeurs rouge / vert / bleu
                    ui.horizontal(|ui| {
                        let mut c = *rgb;
                        let mut ch = false;
                        for (i, n) in [tr("R"), tr("V"), tr("B")].into_iter().enumerate() {
                            ui.label(n);
                            ch |= ui.add(egui::DragValue::new(&mut c[i]).range(0..=255)).changed();
                        }
                        if ch {
                            *rgb = c;
                            st.hsv = rgb_to_hsv(c);
                            st.hex = hex_of(c);
                            changed = true;
                        }
                    });
                    ui.weak(tr("R = rouge  ·  V = vert  ·  B = bleu"));
                });
            });
        let inside = area.response.rect;
        let (clicked, pos, esc) =
            ui.input(|i| (i.pointer.any_click(), i.pointer.interact_pos(), i.key_pressed(Key::Escape)));
        let outside = clicked && pos.map(|p| !inside.contains(p) && !rect.contains(p)).unwrap_or(false);
        if esc || outside {
            st.open = false;
        }
    }
    ui.ctx().data_mut(|d| d.insert_temp(sid, st));
    changed
}
