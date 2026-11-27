//! Zoom et déplacement d'une image, partagés par la visionneuse et l'éditeur intégré.
//!
//! Molette : zoom vers le curseur · glisser : déplacer · double-clic : ajuster / taille réelle ·
//! + et − : zoom · 0 : ajuster à la fenêtre · 1 : taille réelle. Les changements sont animés.
use crate::i18n::{lang, tr, FR};
use eframe::egui::{self, pos2, vec2, Color32, Key, PointerButton, Pos2, Rect, Response, Stroke, Vec2};

const MIN: f32 = 0.02;
const MAX: f32 = 16.0;
/// Paliers des boutons + / − et des touches du clavier.
const STEPS: [f32; 16] = [0.1, 0.25, 0.33, 0.5, 0.67, 0.75, 1.0, 1.25, 1.5, 2.0, 3.0, 4.0, 6.0, 8.0, 12.0, 16.0];

/// Façon de déplacer l'image quand elle dépasse de la zone.
#[derive(Clone, Copy, PartialEq)]
pub enum Pan {
    /// visionneuse : glisser avec le bouton gauche (ou la molette)
    Primary,
    /// éditeur : le bouton gauche dessine, on déplace avec la molette enfoncée ou Espace + glisser
    /// (`space` = vrai quand Espace + glisser est permis, c'est-à-dire hors saisie de texte et hors tracé)
    Middle { space: bool },
}

pub struct Zoom {
    fit: bool,      // l'image s'adapte à la zone (et suit son redimensionnement)
    scale: f32,     // échelle visée
    offset: Vec2,   // décalage visé du centre de l'image par rapport au centre de la zone
    shown_scale: f32,
    shown_offset: Vec2,
    anchor: Option<(Pos2, Vec2)>, // point écran à garder immobile pendant l'animation + point de l'image dessous
    started: bool,
    last_move: f64, // dernier mouvement de la souris au-dessus de l'image (pour afficher les contrôles)
}

impl Default for Zoom {
    fn default() -> Self {
        Self {
            fit: true,
            scale: 1.0,
            offset: Vec2::ZERO,
            shown_scale: 1.0,
            shown_offset: Vec2::ZERO,
            anchor: None,
            started: false,
            last_move: -10.0,
        }
    }
}

fn fit_scale(area: Rect, img: Vec2) -> f32 {
    (area.width() / img.x).min(area.height() / img.y).min(1.0).max(MIN)
}

/// Décalage ramené dans les limites : une image plus petite que la zone reste centrée,
/// une image plus grande ne laisse pas de vide sur les bords.
fn clamp_offset(off: Vec2, area: Rect, img: Vec2, scale: f32) -> Vec2 {
    let size = img * scale;
    let lim = |o: f32, s: f32, a: f32| if s <= a { 0.0 } else { o.clamp(-(s - a) / 2.0, (s - a) / 2.0) };
    vec2(lim(off.x, size.x, area.width()), lim(off.y, size.y, area.height()))
}

impl Zoom {
    /// Revient à l'image entière (animé).
    pub fn fit(&mut self) {
        self.fit = true;
        self.anchor = None;
    }

    /// Zoom vers `new_scale` en gardant immobile le point de l'image situé sous `at`.
    fn zoom_to(&mut self, new_scale: f32, at: Pos2, area: Rect) {
        let new = new_scale.clamp(MIN, MAX);
        let c = area.center();
        let u = (at - c - self.offset) / self.scale;
        self.offset = at - c - u * new;
        self.scale = new;
        self.fit = false;
        self.anchor = Some((at, u));
    }

    fn step(&mut self, dir: i32, area: Rect) {
        let s = self.scale;
        let next = if dir > 0 {
            STEPS.iter().copied().find(|&v| v > s * 1.01).unwrap_or(MAX)
        } else {
            STEPS.iter().rev().copied().find(|&v| v < s * 0.99).unwrap_or(MIN)
        };
        self.zoom_to(next, area.center(), area);
    }

    /// Vrai si le glisser en cours sert à déplacer l'image (et non à dessiner).
    pub fn panning(&self, ui: &egui::Ui, resp: &Response, pan: Pan) -> bool {
        match pan {
            Pan::Primary => resp.dragged_by(PointerButton::Primary) || resp.dragged_by(PointerButton::Middle),
            Pan::Middle { space } => {
                resp.dragged_by(PointerButton::Middle) || (space && resp.dragged_by(PointerButton::Primary) && ui.input(|i| i.key_down(Key::Space)))
            }
        }
    }

    /// Gère la molette, le glisser, le double-clic et le clavier dans `area` (dont `resp` est la réponse),
    /// pour une image de taille `img` (en pixels). Renvoie le rectangle de l'image à l'écran.
    pub fn update(&mut self, ui: &egui::Ui, resp: &Response, area: Rect, img: Vec2, pan: Pan, keys: bool, dbl: bool) -> Rect {
        let ctx = ui.ctx().clone();
        let fit_s = fit_scale(area, img);
        if self.fit {
            self.scale = fit_s;
            self.offset = Vec2::ZERO;
        }
        if !self.started {
            self.shown_scale = self.scale;
            self.shown_offset = self.offset;
            self.started = true;
        }
        let now = ui.input(|i| i.time);
        if resp.hovered() && ui.input(|i| i.pointer.is_moving()) {
            self.last_move = now;
        }

        // molette (et Ctrl + molette, pincement du pavé tactile) : zoom vers le curseur
        if let Some(p) = resp.hover_pos() {
            let (wheel, pinch) = ui.input(|i| (i.raw_scroll_delta.y, i.zoom_delta()));
            let f = pinch * (wheel * 0.0045).exp();
            if (f - 1.0).abs() > 1e-4 {
                self.zoom_to(self.scale * f, p, area);
            }
        }

        // glisser : déplacer l'image (seulement si elle dépasse)
        if self.panning(ui, resp, pan) {
            let d = resp.drag_delta();
            if d != Vec2::ZERO {
                if self.fit {
                    self.fit = false;
                }
                self.anchor = None;
                self.offset += d;
                self.shown_offset += d;
            }
            ctx.set_cursor_icon(egui::CursorIcon::Grabbing);
        } else if resp.hovered() && pan == Pan::Primary && (img.x * self.scale > area.width() + 1.0 || img.y * self.scale > area.height() + 1.0) {
            ctx.set_cursor_icon(egui::CursorIcon::Grab);
        }

        // double-clic : image entière ↔ taille réelle, vers le point cliqué
        if dbl && resp.double_clicked() {
            if let Some(p) = resp.interact_pointer_pos() {
                if self.fit && fit_s < 0.999 {
                    self.zoom_to(1.0, p, area);
                } else {
                    self.fit();
                }
            }
        }

        if keys {
            let (plus, minus, zero, one) = ui.input(|i| {
                (
                    i.key_pressed(Key::Plus) || i.key_pressed(Key::Equals),
                    i.key_pressed(Key::Minus),
                    i.key_pressed(Key::Num0),
                    i.key_pressed(Key::Num1),
                )
            });
            if plus {
                self.step(1, area);
            }
            if minus {
                self.step(-1, area);
            }
            if zero {
                self.fit();
            }
            if one {
                self.zoom_to(1.0, area.center(), area);
            }
        }
        if !self.fit {
            self.offset = clamp_offset(self.offset, area, img, self.scale);
        }

        // animation : l'échelle et la position rejoignent en douceur la cible
        let dt = ui.input(|i| i.stable_dt).min(0.1);
        let k = 1.0 - (-dt * 14.0).exp();
        self.shown_scale += (self.scale - self.shown_scale) * k;
        match self.anchor {
            // le point sous le curseur reste immobile pendant tout le zoom
            Some((at, u)) => {
                let off = at - area.center() - u * self.shown_scale;
                self.shown_offset = clamp_offset(off, area, img, self.shown_scale);
            }
            None => self.shown_offset += (self.offset - self.shown_offset) * k,
        }
        if (self.shown_scale - self.scale).abs() < self.scale * 0.002 && (self.shown_offset - self.offset).length() < 0.5 {
            self.shown_scale = self.scale;
            self.shown_offset = self.offset;
            self.anchor = None;
        } else {
            ctx.request_repaint();
        }
        Rect::from_center_size(area.center() + self.shown_offset, img * self.shown_scale)
    }

    /// Petite barre flottante en bas de la zone : − · pourcentage · + · ajuster.
    /// Elle apparaît quand la souris bouge au-dessus de l'image et s'efface ensuite.
    fn bar_rect(area: Rect) -> Rect {
        let size = vec2(300.0, 42.0);
        Rect::from_center_size(pos2(area.center().x, area.max.y - 18.0 - size.y / 2.0), size)
    }

    /// La souris est-elle sur la barre de zoom (quand elle est affichée) ? L'appelant évite alors
    /// de dessiner ou de cacher la souris dessous.
    pub fn over_controls(&self, ui: &egui::Ui, area: Rect) -> bool {
        let now = ui.input(|i| i.time);
        now - self.last_move < 2.2 && ui.rect_contains_pointer(Self::bar_rect(area).expand(6.0))
    }

    pub fn controls(&mut self, ui: &mut egui::Ui, area: Rect, hint: &str) {
        let ctx = ui.ctx().clone();
        let now = ui.input(|i| i.time);
        let bar = Self::bar_rect(area);
        let over = ui.rect_contains_pointer(bar.expand(12.0));
        if over {
            self.last_move = now;
        }
        let awake = now - self.last_move < 2.2;
        let k = ctx.animate_bool_with_time_and_easing(egui::Id::new("zoom_bar").with(area.min.x as i32), awake, 0.3, egui::emath::easing::cubic_out);
        if awake {
            ctx.request_repaint_after(std::time::Duration::from_millis(400));
        }
        if k < 0.02 {
            return;
        }
        let bar = bar.translate(vec2(0.0, 12.0 * (1.0 - k)));
        let mut p = ui.painter_at(area);
        p.multiply_opacity(k);
        let v = ui.visuals();
        let shadow = egui::epaint::Shadow { offset: vec2(0.0, 6.0), blur: 20.0, spread: 0.0, color: Color32::from_black_alpha(90) };
        p.add(shadow.as_shape(bar, 21.0));
        p.rect_filled(bar, 21.0, v.window_fill.gamma_multiply(0.96));
        p.rect_stroke(bar, 21.0, Stroke::new(1.0_f32, v.widgets.noninteractive.bg_stroke.color));

        let mut cu = ui.new_child(egui::UiBuilder::new().max_rect(bar.shrink2(vec2(10.0, 5.0))).layout(egui::Layout::left_to_right(egui::Align::Center)));
        cu.multiply_opacity(k);
        cu.spacing_mut().item_spacing.x = 4.0;
        cu.spacing_mut().button_padding = vec2(8.0, 4.0);
        let area_c = area;
        if cu.add(egui::Button::new(crate::icons::MINUS).min_size(vec2(34.0, 30.0))).on_hover_text(tr("Zoom arrière (−)")).clicked() {
            self.step(-1, area_c);
        }
        let pct = (self.scale * 100.0).round() as i32;
        let label = if lang() == FR { format!("{pct} %") } else { format!("{pct}%") };
        let next = if (self.scale - 1.0).abs() < 0.001 { tr("Adapter à la fenêtre (0)") } else { tr("Taille réelle (1)") };
        if cu.add(egui::Button::new(label).min_size(vec2(76.0, 30.0))).on_hover_text(format!("{next}\n{hint}")).clicked() {
            if (self.scale - 1.0).abs() < 0.001 {
                self.fit();
            } else {
                self.zoom_to(1.0, area_c.center(), area_c);
            }
        }
        if cu.add(egui::Button::new(crate::icons::PLUS).min_size(vec2(34.0, 30.0))).on_hover_text(tr("Zoom avant (+)")).clicked() {
            self.step(1, area_c);
        }
        cu.separator();
        if cu.add_enabled(!self.fit, egui::Button::new(format!("{}  {}", crate::icons::FIT, tr("Ajuster"))).min_size(vec2(0.0, 30.0))).on_hover_text(tr("Adapter à la fenêtre (0)")).clicked() {
            self.fit();
        }
    }
}
