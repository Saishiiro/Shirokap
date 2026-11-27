use crate::common::*;
use crate::i18n::*;
use crate::icons;
use crate::picker::{color_button, on_accent};
use ab_glyph::{point, Font, FontRef, PxScale, ScaleFont};
use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Key, Pos2, Rect, Sense, Stroke};
use image::RgbaImage;
use std::{
    path::{Path, PathBuf},
    time::Duration,
};

static FONT: &[u8] = include_bytes!("../assets/fonts/Inter-SemiBold.ttf");

#[derive(Clone, Copy, PartialEq)]
enum Tool {
    Pen,
    Shape,
    Highlight,
    Blur,
    Text,
    Step,
    Eraser,
}

#[derive(Clone, Copy, PartialEq)]
enum ShapeKind {
    Rect,
    RectFill,
    Ellipse,
    EllipseFill,
    Line,
    Arrow,
    DoubleArrow,
}

const SHAPES: [(ShapeKind, &str); 7] = [
    (ShapeKind::Rect, "Rectangle"),
    (ShapeKind::RectFill, "Rectangle plein"),
    (ShapeKind::Ellipse, "Ellipse"),
    (ShapeKind::EllipseFill, "Ellipse pleine"),
    (ShapeKind::Line, "Ligne"),
    (ShapeKind::Arrow, "Flèche"),
    (ShapeKind::DoubleArrow, "Flèche double"),
];

enum Op {
    Pen { pts: Vec<Pos2>, col: [u8; 3], w: f32 },
    Shape { kind: ShapeKind, a: Pos2, b: Pos2, col: [u8; 3], w: f32 },
    Highlight { a: Pos2, b: Pos2, col: [u8; 3] },
    Blur { a: Pos2, b: Pos2 },
    Text { pos: Pos2, text: String, col: [u8; 3], size: f32 },
    Step { pos: Pos2, n: u32, col: [u8; 3], size: f32 },
    Erase { pts: Vec<Pos2>, w: f32 }, // gomme : restaure les pixels d'origine
}

pub enum Exit {
    None,
    Close,
    Saved(PathBuf),
}

/// Petit éditeur d'annotations : stylo, rectangle, flèche, surligneur, flou, texte, étapes numérotées.
pub struct Editor {
    path: PathBuf,
    img: RgbaImage,
    base: RgbaImage, // image + toutes les annotations déjà appliquées (ce qu'on voit à l'écran)
    tex: egui::TextureHandle,
    ops: Vec<Op>,
    redo: Vec<Op>, // annotations annulées, qu'on peut rétablir
    cur: Option<Op>,
    tool: Tool,
    shape: ShapeKind,
    col: [u8; 3],
    width: f32,
    ew: f32,              // taille de la gomme
    tsize: f32,           // taille du texte / des pastilles numérotées
    pending: Option<Pos2>, // emplacement du texte en cours de saisie
    buf: String,
    caret_idx: usize, // position du curseur dans le texte en cours de saisie
    status: String,
    confirm: bool, // confirmation avant de remplacer l'original
    dirty: Option<[u32; 4]>, // zone à renvoyer à la carte graphique (gomme)
    zoom: crate::zoom::Zoom, // zoom et déplacement dans l'image
}

fn rgb(c: [u8; 3]) -> Color32 {
    Color32::from_rgb(c[0], c[1], c[2])
}

fn contrast(c: [u8; 3]) -> [u8; 3] {
    let lum = 0.299 * c[0] as f32 + 0.587 * c[1] as f32 + 0.114 * c[2] as f32;
    if lum > 160.0 {
        [0, 0, 0]
    } else {
        [255, 255, 255]
    }
}

fn arrow_head(a: Pos2, b: Pos2, w: f32) -> [Pos2; 2] {
    let ang = (b.y - a.y).atan2(b.x - a.x);
    let len = (w * 4.0).max(18.0);
    [ang + 2.6, ang - 2.6].map(|t| pos2(b.x + len * t.cos(), b.y + len * t.sin()))
}

// ---------- rendu sur les pixels ----------
fn blend(img: &mut RgbaImage, x: i32, y: i32, c: [u8; 3], a: f32) {
    if x < 0 || y < 0 || x >= img.width() as i32 || y >= img.height() as i32 {
        return;
    }
    let p = img.get_pixel_mut(x as u32, y as u32);
    for i in 0..3 {
        p.0[i] = (p.0[i] as f32 * (1.0 - a) + c[i] as f32 * a) as u8;
    }
}

fn disc(img: &mut RgbaImage, cx: f32, cy: f32, r: f32, c: [u8; 3]) {
    for y in (cy - r).floor() as i32..=(cy + r).ceil() as i32 {
        for x in (cx - r).floor() as i32..=(cx + r).ceil() as i32 {
            let (dx, dy) = (x as f32 - cx, y as f32 - cy);
            if dx * dx + dy * dy <= r * r {
                blend(img, x, y, c, 1.0);
            }
        }
    }
}

fn line(img: &mut RgbaImage, a: Pos2, b: Pos2, w: f32, c: [u8; 3]) {
    let r = (w / 2.0).max(0.5);
    let n = a.distance(b).ceil().max(1.0) as i32;
    for i in 0..=n {
        let t = i as f32 / n as f32;
        disc(img, a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t, r, c);
    }
}

/// Gomme : remet les pixels de l'image d'origine dans un disque.
fn erase_disc(img: &mut RgbaImage, orig: &RgbaImage, cx: f32, cy: f32, r: f32) {
    let (w, h) = (img.width() as i32, img.height() as i32);
    let x0 = ((cx - r).floor() as i32).max(0);
    let x1 = ((cx + r).ceil() as i32).min(w - 1);
    let y0 = ((cy - r).floor() as i32).max(0);
    let y1 = ((cy + r).ceil() as i32).min(h - 1);
    for y in y0..=y1 {
        for x in x0..=x1 {
            let (dx, dy) = (x as f32 - cx, y as f32 - cy);
            if dx * dx + dy * dy <= r * r {
                let px = *orig.get_pixel(x as u32, y as u32);
                img.put_pixel(x as u32, y as u32, px);
            }
        }
    }
}

fn erase_line(img: &mut RgbaImage, orig: &RgbaImage, a: Pos2, b: Pos2, w: f32) {
    let r = (w / 2.0).max(0.5);
    let n = a.distance(b).ceil().max(1.0) as i32;
    for i in 0..=n {
        let t = i as f32 / n as f32;
        erase_disc(img, orig, a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t, r);
    }
}

fn bounds(img: &RgbaImage, a: Pos2, b: Pos2) -> (u32, u32, u32, u32) {
    let x0 = a.x.min(b.x).max(0.0) as u32;
    let y0 = a.y.min(b.y).max(0.0) as u32;
    let x1 = (a.x.max(b.x).max(0.0) as u32).min(img.width());
    let y1 = (a.y.max(b.y).max(0.0) as u32).min(img.height());
    (x0, y0, x1, y1)
}

fn fill(img: &mut RgbaImage, a: Pos2, b: Pos2, c: [u8; 3], alpha: f32) {
    let (x0, y0, x1, y1) = bounds(img, a, b);
    for y in y0..y1 {
        for x in x0..x1 {
            blend(img, x as i32, y as i32, c, alpha);
        }
    }
}

fn ellipse_pts(a: Pos2, b: Pos2) -> Vec<Pos2> {
    let c = pos2((a.x + b.x) / 2.0, (a.y + b.y) / 2.0);
    let (rx, ry) = ((b.x - a.x).abs() / 2.0, (b.y - a.y).abs() / 2.0);
    let n = ((rx + ry) * 0.9).clamp(24.0, 400.0) as usize;
    (0..=n)
        .map(|i| {
            let t = i as f32 / n as f32 * std::f32::consts::TAU;
            pos2(c.x + rx * t.cos(), c.y + ry * t.sin())
        })
        .collect()
}

fn fill_ellipse(img: &mut RgbaImage, a: Pos2, b: Pos2, c: [u8; 3]) {
    let (x0, y0, x1, y1) = bounds(img, a, b);
    let (cx, cy) = ((a.x + b.x) / 2.0, (a.y + b.y) / 2.0);
    let rx = ((b.x - a.x).abs() / 2.0).max(0.5);
    let ry = ((b.y - a.y).abs() / 2.0).max(0.5);
    for y in y0..y1 {
        for x in x0..x1 {
            let (dx, dy) = ((x as f32 + 0.5 - cx) / rx, (y as f32 + 0.5 - cy) / ry);
            if dx * dx + dy * dy <= 1.0 {
                blend(img, x as i32, y as i32, c, 1.0);
            }
        }
    }
}

fn pixelate(img: &mut RgbaImage, a: Pos2, b: Pos2) {
    let (x0, y0, x1, y1) = bounds(img, a, b);
    let bs = 14u32;
    let mut y = y0;
    while y < y1 {
        let ye = (y + bs).min(y1);
        let mut x = x0;
        while x < x1 {
            let xe = (x + bs).min(x1);
            let (mut r, mut g, mut bl, mut n) = (0u32, 0u32, 0u32, 0u32);
            for yy in y..ye {
                for xx in x..xe {
                    let p = img.get_pixel(xx, yy).0;
                    r += p[0] as u32;
                    g += p[1] as u32;
                    bl += p[2] as u32;
                    n += 1;
                }
            }
            if n > 0 {
                for yy in y..ye {
                    for xx in x..xe {
                        let p = img.get_pixel_mut(xx, yy);
                        p.0[0] = (r / n) as u8;
                        p.0[1] = (g / n) as u8;
                        p.0[2] = (bl / n) as u8;
                    }
                }
            }
            x += bs;
        }
        y += bs;
    }
}

// ---------- texte (police intégrée à egui, rasterisée avec ab_glyph) ----------

/// Même police (Inter SemiBold) que celle rasterisée dans l'image finale.
fn bf(size: f32) -> FontId {
    FontId::new(size, egui::FontFamily::Name("bold".into()))
}

/// Même échelle qu'egui : `em` = taille de la police en pixels (et non la hauteur totale ascendante + descendante).
fn px(font: &FontRef, em: f32) -> PxScale {
    font.pt_to_px_scale(em).unwrap_or(PxScale::from(em * 1.21))
}

fn text_w(font: &FontRef, size: f32, s: &str) -> f32 {
    let sf = font.as_scaled(px(font, size));
    s.chars().map(|c| sf.h_advance(font.glyph_id(c))).sum()
}

/// Polices de l'image finale : Inter, plus la police chinoise de Windows si le texte contient
/// des caractères absents d'Inter (chinois, japonais…).
fn fonts_for(text: &str) -> Vec<FontRef<'static>> {
    let mut v: Vec<FontRef<'static>> = FontRef::try_from_slice(FONT).ok().into_iter().collect();
    let missing = v.first().map(|f| text.chars().any(|c| !c.is_whitespace() && f.glyph_id(c).0 == 0)).unwrap_or(false);
    if missing {
        if let Some(b) = cjk_font() {
            if let Ok(f) = FontRef::try_from_slice_and_index(b, 0) {
                v.push(f);
            }
        }
    }
    v
}

/// `pos` = coin haut-gauche du texte. `bold` = épaisseur simulée en pixels.
/// Chaque caractère utilise la première police qui le contient.
fn draw_text(img: &mut RgbaImage, fonts: &[FontRef], pos: Pos2, text: &str, size: f32, col: [u8; 3], alpha: f32, bold: i32) {
    let Some(main) = fonts.first() else { return };
    let sf = main.as_scaled(px(main, size));
    let lh = sf.ascent() - sf.descent() + sf.line_gap();
    for (li, ln) in text.lines().enumerate() {
        let mut x = pos.x;
        let y = pos.y + sf.ascent() + li as f32 * lh;
        let mut prev: Option<(usize, ab_glyph::GlyphId)> = None;
        for ch in ln.chars() {
            let fi = fonts.iter().position(|f| f.glyph_id(ch).0 != 0).unwrap_or(0);
            let font = &fonts[fi];
            let scale = px(font, size);
            let fs = font.as_scaled(scale);
            let id = font.glyph_id(ch);
            if let Some((pf, p)) = prev {
                if pf == fi {
                    x += fs.kern(p, id);
                }
            }
            let g = id.with_scale_and_position(scale, point(x, y));
            x += fs.h_advance(id);
            prev = Some((fi, id));
            if let Some(o) = font.outline_glyph(g) {
                let b = o.px_bounds();
                o.draw(|gx, gy, cov| {
                    let (px, py) = (b.min.x as i32 + gx as i32, b.min.y as i32 + gy as i32);
                    for d in 0..=bold {
                        blend(img, px + d, py, col, cov * alpha);
                    }
                });
            }
        }
    }
}

fn apply(img: &mut RgbaImage, orig: &RgbaImage, op: &Op) {
    match op {
        Op::Erase { pts, w } => {
            if pts.len() == 1 {
                erase_disc(img, orig, pts[0].x, pts[0].y, (*w / 2.0).max(0.5));
            }
            for s in pts.windows(2) {
                erase_line(img, orig, s[0], s[1], *w);
            }
        }
        Op::Pen { pts, col, w } => {
            if pts.len() == 1 {
                disc(img, pts[0].x, pts[0].y, (*w / 2.0).max(0.5), *col);
            }
            for s in pts.windows(2) {
                line(img, s[0], s[1], *w, *col);
            }
        }
        Op::Shape { kind, a, b, col, w } => match kind {
            ShapeKind::Rect => {
                let (lo, hi) = (a.min(*b), a.max(*b));
                let c = [lo, pos2(hi.x, lo.y), hi, pos2(lo.x, hi.y), lo];
                for s in c.windows(2) {
                    line(img, s[0], s[1], *w, *col);
                }
            }
            ShapeKind::RectFill => fill(img, *a, *b, *col, 1.0),
            ShapeKind::Ellipse => {
                let pts = ellipse_pts(*a, *b);
                for s in pts.windows(2) {
                    line(img, s[0], s[1], *w, *col);
                }
            }
            ShapeKind::EllipseFill => fill_ellipse(img, *a, *b, *col),
            ShapeKind::Line => line(img, *a, *b, *w, *col),
            ShapeKind::Arrow | ShapeKind::DoubleArrow => {
                line(img, *a, *b, *w, *col);
                for h in arrow_head(*a, *b, *w) {
                    line(img, *b, h, *w, *col);
                }
                if *kind == ShapeKind::DoubleArrow {
                    for h in arrow_head(*b, *a, *w) {
                        line(img, *a, h, *w, *col);
                    }
                }
            }
        },
        Op::Highlight { a, b, col } => fill(img, *a, *b, *col, 0.4),
        Op::Blur { a, b } => pixelate(img, *a, *b),
        Op::Text { pos, text, col, size } => {
            let fonts = fonts_for(text);
            let sh = ((*size / 16.0).round() as f32).max(1.0);
            draw_text(img, &fonts, pos2(pos.x + sh, pos.y + sh), text, *size, [0, 0, 0], 0.55, 0); // ombre
            draw_text(img, &fonts, *pos, text, *size, *col, 1.0, 0);
        }
        Op::Step { pos, n, col, size } => {
            disc(img, pos.x, pos.y, *size * 0.7, *col);
            let fonts = fonts_for("");
            if let Some(f) = fonts.first() {
                let s = n.to_string();
                let sf = f.as_scaled(px(f, *size));
                let w = text_w(f, *size, &s);
                let top = pos.y - (sf.ascent() - sf.descent()) / 2.0;
                draw_text(img, &fonts, pos2(pos.x - w / 2.0, top), &s, *size, contrast(*col), 1.0, 0);
            }
        }
    }
}

// ---------- aperçu à l'écran ----------
fn paint_op(p: &egui::Painter, op: &Op, o: Pos2, s: f32) {
    let sp = |q: Pos2| o + q.to_vec2() * s;
    match op {
        Op::Erase { .. } => {}
        Op::Pen { pts, col, w } => {
            let st = Stroke::new(*w * s, rgb(*col));
            for q in pts {
                p.circle_filled(sp(*q), *w * s / 2.0, rgb(*col));
            }
            for seg in pts.windows(2) {
                p.line_segment([sp(seg[0]), sp(seg[1])], st);
            }
        }
        Op::Shape { kind, a, b, col, w } => {
            let st = Stroke::new(*w * s, rgb(*col));
            let (lo, hi) = (sp(a.min(*b)), sp(a.max(*b)));
            match kind {
                ShapeKind::Rect => {
                    p.rect_stroke(Rect::from_min_max(lo, hi), 0.0, st);
                }
                ShapeKind::RectFill => {
                    p.rect_filled(Rect::from_min_max(lo, hi), 0.0, rgb(*col));
                }
                ShapeKind::Ellipse => {
                    p.add(egui::Shape::ellipse_stroke(lo + (hi - lo) / 2.0, (hi - lo) / 2.0, st));
                }
                ShapeKind::EllipseFill => {
                    p.add(egui::Shape::ellipse_filled(lo + (hi - lo) / 2.0, (hi - lo) / 2.0, rgb(*col)));
                }
                ShapeKind::Line => {
                    p.line_segment([sp(*a), sp(*b)], st);
                }
                ShapeKind::Arrow | ShapeKind::DoubleArrow => {
                    p.line_segment([sp(*a), sp(*b)], st);
                    for h in arrow_head(*a, *b, *w) {
                        p.line_segment([sp(*b), sp(h)], st);
                    }
                    if *kind == ShapeKind::DoubleArrow {
                        for h in arrow_head(*b, *a, *w) {
                            p.line_segment([sp(*a), sp(h)], st);
                        }
                    }
                }
            }
        }
        Op::Highlight { a, b, col } => {
            let c = Color32::from_rgba_unmultiplied(col[0], col[1], col[2], 100);
            p.rect_filled(Rect::from_min_max(sp(a.min(*b)), sp(a.max(*b))), 0.0, c);
        }
        Op::Blur { a, b } => {
            let r = Rect::from_min_max(sp(a.min(*b)), sp(a.max(*b)));
            p.rect_filled(r, 0.0, Color32::from_black_alpha(120));
            p.rect_stroke(r, 0.0, Stroke::new(1.0_f32, Color32::WHITE));
        }
        Op::Text { pos, text, col, size } => {
            p.text(sp(*pos), Align2::LEFT_TOP, text, bf(*size * s), rgb(*col));
        }
        Op::Step { pos, n, col, size } => {
            p.circle_filled(sp(*pos), *size * 0.7 * s, rgb(*col));
            p.text(sp(*pos), Align2::CENTER_CENTER, n.to_string(), bf(*size * s), rgb(contrast(*col)));
        }
    }
}

/// Barre de saisie clignotante (noir + blanc pour rester visible sur tous les fonds).
fn caret(p: &egui::Painter, x: f32, y: f32, h: f32) {
    p.line_segment([pos2(x, y), pos2(x, y + h)], Stroke::new(3.5_f32, Color32::from_black_alpha(170)));
    p.line_segment([pos2(x, y), pos2(x, y + h)], Stroke::new(1.6_f32, Color32::WHITE));
}

/// Bouton de menu déroulant : coins arrondis, surligné quand il est actif, avec une petite flèche « v ».
/// (`Button::selected` d'egui donne des coins carrés, d'où ce bouton maison.)
/// Bouton d'outil de l'éditeur : transparent, teinté au survol, plein de la couleur d'accent quand l'outil
/// est choisi. `chevron` : petite flèche (l'outil ouvre une liste).
fn tool_btn(ui: &mut egui::Ui, selected: bool, icon: &str, label: &str, chevron: bool) -> egui::Response {
    let g = ui.painter().layout_no_wrap(format!("{icon}  {label}"), FontId::proportional(14.5), Color32::PLACEHOLDER);
    let extra = if chevron { 18.0 } else { 0.0 };
    let (rect, resp) = ui.allocate_exact_size(vec2(g.size().x + 26.0 + extra, 34.0), Sense::click());
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let hk = ui.ctx().animate_bool_with_time(resp.id.with("hov"), resp.hovered(), 0.2);
    let sk = ui.ctx().animate_bool_with_time(resp.id.with("sel"), selected, 0.2);
    let v = ui.visuals();
    let acc = crate::gallery::ui_kit::accent_of(v);
    let p = ui.painter();
    if hk * (1.0 - sk) > 0.01 {
        p.rect_filled(rect, 17.0, v.widgets.hovered.weak_bg_fill.gamma_multiply(hk * (1.0 - sk)));
    }
    if sk > 0.01 {
        p.rect_filled(rect, 17.0, acc.gamma_multiply(sk));
    }
    let col = v.text_color().lerp_to_gamma(on_accent(acc), sk);
    p.galley(pos2(rect.min.x + 13.0, rect.center().y - g.size().y / 2.0), g, col);
    if chevron {
        let c = pos2(rect.right() - 16.0, rect.center().y + 1.0);
        let st = Stroke::new(1.7_f32, col);
        p.line_segment([c + vec2(-4.0, -2.0), c + vec2(0.0, 2.0)], st);
        p.line_segment([c + vec2(0.0, 2.0), c + vec2(4.0, -2.0)], st);
    }
    resp
}

/// Petit dessin d'une forme (vignette du menu « Formes »), dans `r`.
fn shape_glyph(p: &egui::Painter, r: Rect, kind: ShapeKind, color: Color32) {
    let st = Stroke::new(1.8_f32, color);
    let (a, b) = (r.left_bottom(), r.right_top());
    // pointe de flèche en `to`, venant de `from`
    let head = |from: Pos2, to: Pos2| {
        let d = (to - from).normalized();
        let n = vec2(-d.y, d.x);
        p.line_segment([to, to - d * 6.0 + n * 4.0], st);
        p.line_segment([to, to - d * 6.0 - n * 4.0], st);
    };
    match kind {
        ShapeKind::Rect => {
            p.rect_stroke(r, 2.5, st);
        }
        ShapeKind::RectFill => {
            p.rect_filled(r, 2.5, color);
        }
        ShapeKind::Ellipse => {
            p.add(egui::Shape::ellipse_stroke(r.center(), r.size() / 2.0, st));
        }
        ShapeKind::EllipseFill => {
            p.add(egui::Shape::ellipse_filled(r.center(), r.size() / 2.0, color));
        }
        ShapeKind::Line => {
            p.line_segment([a, b], st);
        }
        ShapeKind::Arrow => {
            p.line_segment([a, b], st);
            head(a, b);
        }
        ShapeKind::DoubleArrow => {
            p.line_segment([a, b], st);
            head(a, b);
            head(b, a);
        }
    }
}

/// Ligne du menu « Formes » : vignette de la forme, son nom, une coche si elle est choisie.
fn shape_row(ui: &mut egui::Ui, w: f32, selected: bool, kind: ShapeKind, label: &str) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(w, 34.0), Sense::click());
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let hk = ui.ctx().animate_bool_with_time(resp.id.with("hov"), resp.hovered(), 0.15);
    let v = ui.visuals();
    let p = ui.painter();
    let fg = if selected { v.selection.stroke.color } else { v.text_color() };
    if selected {
        p.rect_filled(rect, 9.0, v.selection.bg_fill);
    } else if hk > 0.01 {
        p.rect_filled(rect, 9.0, v.widgets.hovered.weak_bg_fill.gamma_multiply(hk));
    }
    let glyph = Rect::from_center_size(pos2(rect.min.x + 22.0, rect.center().y), vec2(20.0, 14.0));
    shape_glyph(p, glyph, kind, fg);
    p.text(pos2(rect.min.x + 44.0, rect.center().y), Align2::LEFT_CENTER, label, FontId::proportional(14.5), fg);
    if selected {
        crate::gallery::ui_kit::icon_check(p, pos2(rect.right() - 18.0, rect.center().y), fg);
    }
    resp
}

pub fn dropdown(ui: &mut egui::Ui, label: &str, active: bool, add: impl FnOnce(&mut egui::Ui)) {
    let acc = crate::gallery::ui_kit::accent_of(ui.visuals());
    // espaces insécables en fin de libellé : ils réservent la place de la flèche dessinée ensuite
    let txt = format!("{label}\u{00A0}\u{00A0}\u{00A0}\u{00A0}\u{00A0}");
    let mut btn = if active {
        egui::Button::new(egui::RichText::new(txt).color(on_accent(acc))).fill(acc).stroke(Stroke::NONE)
    } else {
        egui::Button::new(txt).stroke(Stroke::NONE)
    };
    btn = btn.min_size(vec2(0.0, 36.0)).rounding(18.0);
    let r = egui::menu::menu_custom_button(ui, btn, add);
    let rect = r.response.rect;
    let c = pos2(rect.right() - 20.0, rect.center().y + 1.0);
    let col = if active { on_accent(acc) } else { ui.visuals().text_color() };
    let st = Stroke::new(1.7_f32, col);
    ui.painter().line_segment([c + vec2(-4.0, -2.0), c + vec2(0.0, 2.0)], st);
    ui.painter().line_segment([c + vec2(0.0, 2.0), c + vec2(4.0, -2.0)], st);
}

impl Editor {
    pub fn open(ctx: &egui::Context, path: &Path) -> Option<Editor> {
        let img = image::open(path).ok()?.to_rgba8();
        let ci = egui::ColorImage::from_rgba_unmultiplied([img.width() as usize, img.height() as usize], img.as_raw());
        let tex = ctx.load_texture("editor", ci, Default::default());
        // taille de texte par défaut proportionnelle à l'image (lisible en 1080p comme en 4K)
        let tsize = (img.width() as f32 / 45.0).clamp(18.0, 90.0);
        Some(Editor {
            path: path.to_path_buf(),
            base: img.clone(),
            img,
            tex,
            ops: Vec::new(),
            redo: Vec::new(),
            cur: None,
            tool: Tool::Pen,
            shape: ShapeKind::Rect,
            col: [239, 68, 68],
            width: 6.0,
            ew: 24.0,
            tsize,
            pending: None,
            buf: String::new(),
            caret_idx: 0,
            status: String::new(),
            confirm: false,
            dirty: None,
            zoom: Default::default(),
        })
    }

    /// Image finale = image d'origine + toutes les annotations (identique à ce qu'on voit).
    fn render(&self) -> RgbaImage {
        self.base.clone()
    }

    fn upload_full(&mut self) {
        let (w, h) = (self.base.width() as usize, self.base.height() as usize);
        let ci = egui::ColorImage::from_rgba_unmultiplied([w, h], self.base.as_raw());
        self.tex.set(ci, Default::default());
    }

    /// Renvoie seulement un rectangle [x0,x1) × [y0,y1) à la carte graphique (rapide, pour la gomme).
    fn upload_region(&mut self, x0: u32, y0: u32, x1: u32, y1: u32) {
        let (w, h) = (x1.saturating_sub(x0) as usize, y1.saturating_sub(y0) as usize);
        if w == 0 || h == 0 {
            return;
        }
        let mut buf = Vec::with_capacity(w * h * 4);
        for y in y0..y1 {
            for x in x0..x1 {
                buf.extend_from_slice(&self.base.get_pixel(x, y).0);
            }
        }
        let ci = egui::ColorImage::from_rgba_unmultiplied([w, h], &buf);
        self.tex.set_partial([x0 as usize, y0 as usize], ci, Default::default());
    }

    fn mark_dirty(&mut self, a: Pos2, b: Pos2, r: f32) {
        let (w, h) = (self.base.width() as f32, self.base.height() as f32);
        let x0 = (a.x.min(b.x) - r - 1.0).floor().clamp(0.0, w) as u32;
        let y0 = (a.y.min(b.y) - r - 1.0).floor().clamp(0.0, h) as u32;
        let x1 = (a.x.max(b.x) + r + 1.0).ceil().clamp(0.0, w) as u32;
        let y1 = (a.y.max(b.y) + r + 1.0).ceil().clamp(0.0, h) as u32;
        self.dirty = Some(match self.dirty {
            Some([a0, b0, a1, b1]) => [a0.min(x0), b0.min(y0), a1.max(x1), b1.max(y1)],
            None => [x0, y0, x1, y1],
        });
    }

    /// Ajoute une annotation. `applied` = déjà dessinée sur l'image (cas de la gomme, appliquée en direct).
    fn commit(&mut self, op: Op, applied: bool) {
        if !applied {
            apply(&mut self.base, &self.img, &op);
            self.upload_full();
        }
        self.ops.push(op);
        self.redo.clear(); // une nouvelle annotation remplace ce qu'on aurait pu rétablir
    }

    /// Recalcule l'image depuis l'original en rejouant toutes les annotations.
    fn rebuild(&mut self) {
        self.base = self.img.clone();
        for op in &self.ops {
            apply(&mut self.base, &self.img, op);
        }
        self.upload_full();
    }

    fn undo(&mut self) {
        if let Some(op) = self.ops.pop() {
            self.redo.push(op);
            self.rebuild();
        }
    }

    /// Remet la dernière annotation annulée.
    fn redo(&mut self) {
        if let Some(op) = self.redo.pop() {
            apply(&mut self.base, &self.img, &op);
            self.upload_full();
            self.ops.push(op);
        }
    }

    fn copy_path(&self) -> PathBuf {
        let stem = self.path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        let ext = self.path.extension().and_then(|e| e.to_str()).unwrap_or("png");
        let mut p = self.path.with_file_name(format!("{stem}_edit.{ext}"));
        let mut n = 2;
        while p.exists() {
            p = self.path.with_file_name(format!("{stem}_edit{n}.{ext}"));
            n += 1;
        }
        p
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) -> Exit {
        let mut exit = Exit::None;
        let mut save: Option<bool> = None; // Some(false) = copie, Some(true) = remplacer l'original
        let mut undo = false;
        let mut redo = false;
        // la barre d'outils garde sa place : si la fenêtre est trop étroite, elle est coupée au lieu d'élargir la toile
        let (can_undo, can_redo) = (!self.ops.is_empty(), !self.redo.is_empty());
        let (mut copy, mut confirm) = (false, false);
        let acc = crate::gallery::ui_kit::accent_of(ui.visuals());
        // ligne 1 : Fermer et les outils à gauche ; historique, copie et enregistrement calés à droite
        crate::gallery::ui_kit::contained(ui, |ui| {
            use crate::gallery::form::{ghost, pill, split_row};
            ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
            let ic = |i: &str, t: &str| format!("{i}  {t}");
            split_row(
                ui,
                "editor_bar",
                |ui| {
                    if ghost(ui, icons::BACK, tr("← Fermer").trim_start_matches("← ")).clicked() {
                        exit = Exit::Close;
                    }
                    // les outils, regroupés dans une pilule en retrait ; l'outil choisi est plein de la couleur d'accent
                    let inset = ui.visuals().extreme_bg_color;
                    egui::Frame::none().fill(inset).rounding(21.0).inner_margin(egui::Margin::same(4.0)).show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing = vec2(2.0, 2.0);
                            if tool_btn(ui, self.tool == Tool::Pen, icons::PEN, tr("Stylo"), false).clicked() {
                                self.tool = Tool::Pen;
                            }
                            let cur_shape = SHAPES.iter().find(|s| s.0 == self.shape).map(|s| tr(s.1)).unwrap_or(tr("Rectangle"));
                            let shapes_label = if self.tool == Tool::Shape { trf("Formes : {0}", &[&cur_shape]) } else { tr("Formes").to_string() };
                            let sr = tool_btn(ui, self.tool == Tool::Shape, icons::SHAPES, &shapes_label, true);
                            let pop = ui.make_persistent_id("shapes_popup");
                            if sr.clicked() {
                                ui.memory_mut(|m| m.toggle_popup(pop));
                            }
                            egui::popup::popup_below_widget(ui, pop, &sr, egui::PopupCloseBehavior::CloseOnClick, |ui| {
                                ui.spacing_mut().item_spacing.y = 2.0;
                                // largeur : le plus long des noms + la vignette de la forme + la coche
                                let w = SHAPES
                                    .iter()
                                    .map(|s| ui.painter().layout_no_wrap(tr(s.1).to_string(), FontId::proportional(14.5), Color32::PLACEHOLDER).size().x)
                                    .fold(0.0_f32, f32::max)
                                    + 92.0;
                                let w = w.max(200.0);
                                ui.set_min_width(w);
                                for (k, name) in SHAPES {
                                    if shape_row(ui, w, self.tool == Tool::Shape && self.shape == k, k, tr(name)).clicked() {
                                        self.shape = k;
                                        self.tool = Tool::Shape;
                                    }
                                }
                            });
                            for (t, i, n) in [
                                (Tool::Highlight, icons::HIGHLIGHT, tr("Surligneur")),
                                (Tool::Blur, icons::BLUR, tr("Flou")),
                                (Tool::Text, icons::TEXT, tr("Texte")),
                                (Tool::Step, icons::STEP, tr("Étape n°")),
                                (Tool::Eraser, icons::ERASER, tr("Gomme")),
                            ] {
                                if tool_btn(ui, self.tool == t, i, n, false).clicked() {
                                    self.tool = t;
                                }
                            }
                        });
                    });
                },
                |ui| {
                    if ui.add_enabled_ui(can_undo, |ui| pill(ui, icons::UNDO, tr("Annuler|modif"))).inner.on_hover_text(tr("Annuler (Ctrl+Z)")).clicked() {
                        undo = true;
                    }
                    if ui.add_enabled_ui(can_redo, |ui| pill(ui, icons::REDO, tr("Rétablir"))).inner.on_hover_text(tr("Rétablir (Ctrl+Y)")).clicked() {
                        redo = true;
                    }
                    if pill(ui, icons::COPY, tr("Copier")).on_hover_text(tr("Copier l'image annotée dans le presse-papiers")).clicked() {
                        copy = true;
                    }
                    let save_btn = egui::Button::new(egui::RichText::new(ic(icons::SAVE, tr("Enregistrer"))).color(on_accent(acc)).family(egui::FontFamily::Name("bold".into())))
                        .fill(acc)
                        .stroke(Stroke::NONE)
                        .rounding(18.0)
                        .min_size(vec2(0.0, 36.0));
                    if ui.add(save_btn).on_hover_text(tr("Enregistre une copie annotée (l'original est conservé)")).clicked() {
                        save = Some(false);
                    }
                    dropdown(ui, &ic(icons::MORE, tr("Plus…")), false, |ui| {
                        if ui.button(tr("Remplacer l'original")).clicked() {
                            confirm = true;
                            ui.close_menu();
                        }
                    });
                },
            );
        });
        if copy {
            copy_to_clipboard(&self.render());
            self.status = format!("{}  {}", icons::COPY, tr("Copiée dans le presse-papiers"));
        }
        if confirm {
            self.confirm = true;
        }
        if self.pending.is_none() {
            // Ctrl+Z : annuler · Ctrl+Y ou Ctrl+Maj+Z : rétablir
            let (z, y, shift) = ui.input(|i| (i.modifiers.command && i.key_pressed(Key::Z), i.modifiers.command && i.key_pressed(Key::Y), i.modifiers.shift));
            if z && !shift {
                undo = true;
            }
            if y || (z && shift) {
                redo = true;
            }
        }
        if undo {
            self.undo();
        } else if redo {
            self.redo();
        }

        // ligne 2 : couleur et taille de l'outil, puis l'aide (ou la saisie du texte à l'emplacement cliqué)
        let (mut commit, mut cancel) = (false, false);
        crate::gallery::ui_kit::contained(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
                if !matches!(self.tool, Tool::Eraser | Tool::Blur) {
                    color_button(ui, "annot", &mut self.col);
                }
                let weak = ui.visuals().weak_text_color();
                let size = match self.tool {
                    Tool::Text | Tool::Step => Some((tr("Taille"), &mut self.tsize, 12.0..=160.0)),
                    Tool::Eraser => Some((tr("Taille"), &mut self.ew, 4.0..=120.0)),
                    Tool::Highlight | Tool::Blur => None,
                    _ => Some((tr("épaisseur"), &mut self.width, 2.0..=30.0)),
                };
                if let Some((name, v, range)) = size {
                    let mut c = name.chars();
                    let name = c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default();
                    ui.label(egui::RichText::new(name).color(weak));
                    ui.spacing_mut().slider_width = 130.0;
                    ui.add(egui::Slider::new(v, range).show_value(false));
                    ui.label(egui::RichText::new(format!("{:.0}", *v)).color(weak));
                }
                ui.add_space(12.0);
                if self.pending.is_some() {
                    ui.label(tr("Texte :"));
                    let out = egui::TextEdit::singleline(&mut self.buf).desired_width(340.0).margin(vec2(12.0, 9.0)).show(ui);
                    let r = out.response;
                    if let Some(cr) = out.cursor_range {
                        self.caret_idx = cr.primary.ccursor.index; // suit les flèches, Début/Fin, la souris…
                    }
                    r.request_focus();
                    if r.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
                        commit = true;
                    }
                    if ui.add(egui::Button::new(egui::RichText::new(tr("Ajouter")).color(on_accent(acc))).fill(acc).stroke(Stroke::NONE).rounding(18.0).min_size(vec2(0.0, 36.0))).clicked() {
                        commit = true;
                    }
                    if crate::gallery::form::ghost(ui, "", tr("Annuler")).clicked() {
                        cancel = true;
                    }
                } else {
                    let hint = if self.status.is_empty() {
                        tr("Glisse sur l'image pour annoter · Texte et Étape n° : clique à l'endroit voulu · Gomme : repasse sur une annotation pour l'effacer")
                    } else {
                        self.status.as_str()
                    };
                    ui.label(egui::RichText::new(hint).size(13.0).color(weak));
                }
            });
        });
        if commit {
            if let Some(q) = self.pending.take() {
                if !self.buf.trim().is_empty() {
                    let op = Op::Text { pos: q, text: self.buf.clone(), col: self.col, size: self.tsize };
                    self.commit(op, false);
                }
            }
            self.buf.clear();
            self.caret_idx = 0;
        }
        if cancel {
            self.pending = None;
            self.buf.clear();
            self.caret_idx = 0;
        }

        if self.confirm {
            let (mut yes, mut no) = (false, false);
            egui::Window::new(tr("Remplacer l'original ?"))
                .collapsible(false)
                .resizable(false)
                .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
                .show(ui.ctx(), |ui| {
                    ui.label(tr("L'image d'origine sera écrasée par la version annotée."));
                    ui.label(tr("C'est définitif. Pour garder l'original, utilise plutôt « Enregistrer »."));
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        let danger = egui::Button::new(egui::RichText::new(tr("Remplacer")).color(Color32::WHITE))
                            .fill(Color32::from_rgb(220, 60, 60));
                        if ui.add(danger).clicked() {
                            yes = true;
                        }
                        if ui.button(tr("Annuler")).clicked() {
                            no = true;
                        }
                    });
                });
            if ui.input(|i| i.key_pressed(Key::Escape)) {
                no = true;
            }
            if yes {
                self.confirm = false;
                save = Some(true);
            }
            if no {
                self.confirm = false;
            }
        }

        // zone de dessin : toute la place restante ; l'image s'y affiche avec le zoom
        // (molette : zoom · clic molette ou Espace + glisser : déplacer · + − 0 1 au clavier)
        let isz = vec2(self.img.width() as f32, self.img.height() as f32);
        let (area, resp) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
        let pan = crate::zoom::Pan::Middle { space: self.cur.is_none() && self.pending.is_none() };
        let keys = self.pending.is_none() && !ui.ctx().wants_keyboard_input();
        let rect = self.zoom.update(ui, &resp, area, isz, pan, keys, false);
        let scale = rect.width() / isz.x;
        let to_img = |s: Pos2| ((s - rect.min) / scale).to_pos2();
        let panning = self.zoom.panning(ui, &resp, pan);
        let space = ui.input(|i| i.key_down(Key::Space)) && self.pending.is_none() && self.cur.is_none();
        // la barre de zoom flotte au-dessus de l'image : sous elle, on ne dessine pas et la souris reste normale
        let over_bar = self.zoom.over_controls(ui, area);
        let on_img = |p: Pos2| rect.contains(p) && !over_bar;

        // ----- clics simples -----
        if resp.clicked() && !space {
            if let Some(pos) = resp.interact_pointer_pos().filter(|&p| on_img(p)) {
                let q = to_img(pos);
                match self.tool {
                    Tool::Text => {
                        self.pending = Some(q);
                        self.buf.clear();
                        self.caret_idx = 0;
                    }
                    Tool::Step => {
                        let n = self.ops.iter().filter(|o| matches!(o, Op::Step { .. })).count() as u32 + 1;
                        let op = Op::Step { pos: q, n, col: self.col, size: self.tsize };
                        self.commit(op, false);
                    }
                    Tool::Pen => {
                        let op = Op::Pen { pts: vec![q], col: self.col, w: self.width };
                        self.commit(op, false);
                    }
                    Tool::Eraser => {
                        let ew = self.ew;
                        erase_disc(&mut self.base, &self.img, q.x, q.y, ew / 2.0);
                        self.mark_dirty(q, q, ew / 2.0);
                        self.commit(Op::Erase { pts: vec![q], w: ew }, true);
                    }
                    _ => {}
                }
            }
        }
        // ----- glisser (bouton gauche ; la molette enfoncée et Espace servent à déplacer l'image) -----
        if resp.drag_started_by(egui::PointerButton::Primary) && !space && !panning {
            if let Some(pos) = resp.interact_pointer_pos().filter(|&p| on_img(p)) {
                let q = to_img(pos);
                let (col, w) = (self.col, self.width);
                match self.tool {
                    Tool::Pen => self.cur = Some(Op::Pen { pts: vec![q], col, w }),
                    Tool::Shape => self.cur = Some(Op::Shape { kind: self.shape, a: q, b: q, col, w }),
                    Tool::Highlight => self.cur = Some(Op::Highlight { a: q, b: q, col }),
                    Tool::Blur => self.cur = Some(Op::Blur { a: q, b: q }),
                    Tool::Eraser => {
                        let ew = self.ew;
                        erase_disc(&mut self.base, &self.img, q.x, q.y, ew / 2.0);
                        self.mark_dirty(q, q, ew / 2.0);
                        self.cur = Some(Op::Erase { pts: vec![q], w: ew });
                    }
                    Tool::Text | Tool::Step => {}
                }
            }
        }
        if resp.dragged_by(egui::PointerButton::Primary) && self.cur.is_some() {
            if let Some(pos) = resp.interact_pointer_pos() {
                let q = to_img(pos);
                let mut stamp: Option<(Pos2, f32)> = None;
                match &mut self.cur {
                    Some(Op::Pen { pts, .. }) => pts.push(q),
                    Some(Op::Shape { b, .. })
                    | Some(Op::Highlight { b, .. })
                    | Some(Op::Blur { b, .. }) => *b = q,
                    Some(Op::Erase { pts, w }) => {
                        let last = pts.last().copied().unwrap_or(q);
                        stamp = Some((last, *w));
                        pts.push(q);
                    }
                    _ => {}
                }
                if let Some((last, w)) = stamp {
                    erase_line(&mut self.base, &self.img, last, q, w); // la gomme agit en direct
                    self.mark_dirty(last, q, w / 2.0);
                }
            }
        }
        if resp.drag_stopped_by(egui::PointerButton::Primary) {
            if let Some(op) = self.cur.take() {
                let applied = matches!(op, Op::Erase { .. });
                self.commit(op, applied);
            }
        }
        if let Some([x0, y0, x1, y1]) = self.dirty.take() {
            self.upload_region(x0, y0, x1, y1);
        }

        // ----- affichage -----
        let p = ui.painter_at(area.intersect(rect));
        p.image(self.tex.id(), rect, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
        if let Some(op) = &self.cur {
            if !matches!(op, Op::Erase { .. }) {
                paint_op(&p, op, rect.min, scale); // aperçu du tracé en cours
            }
        }
        let blink = (ui.input(|i| i.time) * 1.8) as i64 % 2 == 0;
        if let Some(q) = self.pending {
            let sp = rect.min + q.to_vec2() * scale;
            p.circle_filled(sp, 3.0, Color32::WHITE);
            p.text(sp, Align2::LEFT_TOP, &self.buf, bf(self.tsize * scale), rgb(self.col));
            // barre clignotante juste après le texte saisi
            // la barre se place après les lettres situées avant le curseur du champ de saisie
            let prefix: String = self.buf.chars().take(self.caret_idx).collect();
            let g = ui.fonts(|f| f.layout_no_wrap(prefix, bf(self.tsize * scale), rgb(self.col)));
            ui.ctx().request_repaint_after(Duration::from_millis(280));
            if blink {
                caret(&p, sp.x + g.size().x + 1.0, sp.y, self.tsize * scale * 1.2);
            }
        }

        // curseur : rond à la taille du pinceau (stylo, gomme, pastille…) ou barre clignotante pour le texte
        let brush = match self.tool {
            Tool::Pen | Tool::Shape => Some(self.width / 2.0),
            Tool::Eraser => Some(self.ew / 2.0),
            Tool::Step => Some(self.tsize * 0.7),
            _ => None,
        };
        if space && resp.hovered() && !panning {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grab); // Espace enfoncé : prêt à déplacer
        } else if resp.hovered() && !panning && resp.hover_pos().map(on_img).unwrap_or(false) {
            ui.ctx().set_cursor_icon(match self.tool {
                Tool::Pen | Tool::Eraser | Tool::Step | Tool::Text => egui::CursorIcon::None,
                _ => egui::CursorIcon::Crosshair,
            });
            if let Some(pos) = resp.hover_pos() {
                if let Some(r) = brush {
                    let rr = (r * scale).max(2.0);
                    if self.tool != Tool::Eraser {
                        let c = self.col;
                        p.circle_filled(pos, rr, Color32::from_rgba_unmultiplied(c[0], c[1], c[2], 90));
                    }
                    p.circle_stroke(pos, rr + 1.0, Stroke::new(1.2_f32, Color32::from_black_alpha(170)));
                    p.circle_stroke(pos, rr, Stroke::new(1.5_f32, Color32::WHITE));
                    p.circle_filled(pos, 1.5, Color32::WHITE);
                }
                if self.tool == Tool::Text && self.pending.is_none() {
                    ui.ctx().request_repaint_after(Duration::from_millis(280));
                    if blink {
                        caret(&p, pos.x, pos.y, self.tsize * scale * 1.2);
                    }
                }
            }
        }

        self.zoom.controls(ui, area, tr("Molette : zoom · clic molette ou Espace + glisser : déplacer"));

        if let Some(replace) = save {
            let img = self.render();
            let dest = if replace { self.path.clone() } else { self.copy_path() };
            let ext = dest.extension().and_then(|e| e.to_str()).unwrap_or("png").to_lowercase();
            let ok = if ext == "jpg" || ext == "jpeg" {
                save_jpeg(&img, &dest, 92).is_some()
            } else {
                save_png(&img, &dest, 0).is_some()
            };
            if ok {
                make_thumb(&img, &thumb_path(&dest));
                exit = Exit::Saved(dest);
            } else {
                self.status = format!("{}  {}", icons::ALERT, tr("Échec de l'enregistrement"));
            }
        }
        exit
    }
}
