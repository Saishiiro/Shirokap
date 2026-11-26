// Thème, palettes et petits composants d'interface de la galerie.
use super::*;

// ---------- helpers ----------
pub(crate) fn uv() -> Rect {
    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0))
}
pub(crate) fn accent(c: &Config) -> Color32 {
    Color32::from_rgb(c.accent[0], c.accent[1], c.accent[2])
}

pub(crate) fn col(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}

pub(crate) fn bold_text(s: &str, size: f32) -> egui::RichText {
    egui::RichText::new(s).size(size).family(egui::FontFamily::Name("bold".into()))
}

/// Polices Inter (Regular + SemiBold) intégrées à l'exe ; les polices d'egui restent en secours (symboles, emojis).
pub(crate) fn install_fonts(ctx: &egui::Context) {
    install_fonts_with(ctx, lang() == ZH);
}

/// `cjk` : ajoute la police chinoise de Windows en secours (interface en chinois, liste des langues,
/// éditeur). Lue sur le disque seulement la première fois qu'on en a besoin.
pub(crate) fn install_fonts_with(ctx: &egui::Context, cjk: bool) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "Inter".to_owned(),
        egui::FontData::from_static(include_bytes!("../../assets/fonts/Inter-Regular.ttf")).into(),
    );
    fonts.font_data.insert(
        "Inter-SemiBold".to_owned(),
        egui::FontData::from_static(include_bytes!("../../assets/fonts/Inter-SemiBold.ttf")).into(),
    );
    let prop = fonts.families.entry(egui::FontFamily::Proportional).or_default();
    prop.insert(0, "Inter".to_owned());
    if cjk {
        if let Some(b) = cjk_font() {
            fonts.font_data.insert("CJK".to_owned(), egui::FontData::from_static(b));
            prop.push("CJK".to_owned());
        }
    }
    let mut bold = vec!["Inter-SemiBold".to_owned()];
    bold.extend(prop.iter().cloned());
    fonts.families.insert(egui::FontFamily::Name("bold".into()), bold);
    ctx.set_fonts(fonts);
}

/// Palette d'un thème (l'accent reste choisi séparément).
#[derive(Clone, Copy)]
pub(crate) struct Pal {
    pub(crate) dark: bool,
    pub(crate) bg: Color32,
    pub(crate) side: Color32,
    pub(crate) card: Color32,
    pub(crate) input: Color32,
    pub(crate) button: Color32,
    pub(crate) hover: Color32,
    pub(crate) line: Color32,
    pub(crate) stroke_in: Color32,
    pub(crate) stroke_hov: Color32,
    pub(crate) text: Color32,
}

pub(crate) const THEMES: [(&str, &str); 17] = [
    ("dark", "Sombre"),
    ("graphite", "Graphite"),
    ("midnight", "Minuit"),
    ("oled", "Noir"),
    ("ocean", "Océan"),
    ("forest", "Forêt"),
    ("dusk", "Crépuscule"),
    ("ruby", "Rubis"),
    ("glacier", "Glacier"),
    ("mocha", "Moka"),
    ("neon", "Néon"),
    ("light", "Clair"),
    ("sand", "Sable"),
    ("mint", "Menthe"),
    ("lavender", "Lavande"),
    ("sakura", "Sakura"),
    ("sky", "Ciel"),
];

pub(crate) const ACCENTS: [[u8; 3]; 10] = [
    [59, 130, 246],
    [6, 182, 212],
    [34, 197, 94],
    [234, 179, 8],
    [249, 115, 22],
    [239, 68, 68],
    [236, 72, 153],
    [139, 92, 246],
    [255, 255, 255],
    [0, 0, 0],
];

pub(crate) fn palette(theme: &str) -> Pal {
    // (sombre ?, fond, barre latérale, carte, champ, bouton, survol, trait, contour, contour survol, texte)
    let mk = |dark: bool, bg: [u8; 3], side: [u8; 3], card: [u8; 3], input: [u8; 3], button: [u8; 3], hover: [u8; 3], line: [u8; 3], si: [u8; 3], sh: [u8; 3], text: [u8; 3]| Pal {
        dark,
        bg: col(bg[0], bg[1], bg[2]),
        side: col(side[0], side[1], side[2]),
        card: col(card[0], card[1], card[2]),
        input: col(input[0], input[1], input[2]),
        button: col(button[0], button[1], button[2]),
        hover: col(hover[0], hover[1], hover[2]),
        line: col(line[0], line[1], line[2]),
        stroke_in: col(si[0], si[1], si[2]),
        stroke_hov: col(sh[0], sh[1], sh[2]),
        text: col(text[0], text[1], text[2]),
    };
    match theme {
        "graphite" => mk(true, [30, 30, 30], [23, 23, 23], [38, 38, 38], [24, 24, 24], [50, 50, 50], [62, 62, 62], [56, 56, 56], [74, 74, 74], [104, 104, 104], [230, 230, 230]),
        "midnight" => mk(true, [15, 20, 36], [10, 14, 26], [22, 29, 50], [10, 14, 26], [31, 41, 70], [42, 54, 88], [36, 47, 78], [54, 68, 108], [82, 100, 150], [226, 232, 246]),
        "oled" => mk(true, [0, 0, 0], [8, 8, 9], [16, 16, 18], [10, 10, 12], [28, 28, 32], [40, 40, 44], [36, 36, 40], [58, 58, 64], [90, 90, 98], [236, 236, 238]),
        "ocean" => mk(true, [14, 27, 33], [9, 19, 24], [20, 38, 46], [9, 19, 24], [28, 52, 62], [36, 66, 78], [34, 60, 72], [50, 86, 100], [76, 120, 138], [224, 240, 244]),
        "forest" => mk(true, [20, 29, 24], [14, 21, 17], [27, 40, 33], [14, 21, 17], [37, 55, 45], [46, 68, 56], [42, 62, 51], [60, 86, 70], [88, 120, 100], [228, 240, 232]),
        "dusk" => mk(true, [27, 22, 38], [20, 16, 30], [36, 30, 52], [20, 16, 30], [50, 42, 74], [62, 52, 92], [56, 47, 82], [78, 66, 112], [110, 96, 150], [236, 230, 248]),
        "ruby" => mk(true, [33, 16, 19], [24, 11, 13], [45, 23, 27], [24, 11, 13], [62, 32, 38], [84, 44, 52], [74, 38, 45], [102, 56, 66], [140, 80, 92], [246, 230, 232]),
        "glacier" => mk(true, [36, 41, 51], [30, 34, 43], [46, 52, 64], [30, 34, 43], [59, 66, 82], [70, 79, 98], [62, 70, 86], [78, 88, 108], [112, 126, 152], [229, 233, 240]),
        "mocha" => mk(true, [30, 24, 21], [23, 18, 16], [41, 33, 29], [23, 18, 16], [56, 45, 39], [72, 58, 50], [62, 50, 43], [86, 70, 60], [120, 98, 84], [241, 231, 221]),
        "neon" => mk(true, [13, 11, 24], [9, 7, 18], [22, 18, 40], [9, 7, 18], [34, 26, 62], [50, 38, 92], [44, 31, 84], [78, 54, 142], [122, 86, 212], [237, 229, 255]),
        "sakura" => mk(false, [252, 243, 246], [247, 232, 238], [255, 250, 252], [255, 250, 252], [246, 228, 235], [238, 211, 222], [240, 218, 227], [226, 190, 204], [196, 140, 162], [62, 36, 47]),
        "sky" => mk(false, [240, 246, 252], [228, 238, 250], [252, 254, 255], [252, 254, 255], [226, 236, 248], [207, 223, 244], [214, 228, 244], [186, 206, 232], [134, 164, 204], [24, 40, 62]),
        "mint" => mk(false, [239, 247, 243], [228, 240, 234], [250, 255, 252], [250, 255, 252], [226, 240, 233], [208, 229, 218], [209, 226, 217], [186, 210, 197], [140, 175, 157], [26, 48, 38]),
        "lavender" => mk(false, [244, 241, 250], [234, 229, 245], [252, 250, 255], [252, 250, 255], [232, 227, 245], [214, 206, 236], [218, 211, 236], [196, 186, 225], [156, 142, 200], [40, 34, 64]),
        "light" => mk(false, [246, 247, 249], [237, 239, 243], [255, 255, 255], [255, 255, 255], [234, 237, 241], [223, 227, 233], [221, 225, 231], [203, 208, 216], [168, 174, 185], [28, 30, 35]),
        "sand" => mk(false, [247, 242, 233], [238, 231, 218], [255, 252, 246], [255, 252, 246], [240, 232, 218], [232, 222, 205], [228, 219, 203], [212, 200, 180], [180, 166, 140], [52, 45, 36]),
        _ => mk(true, [24, 26, 31], [19, 21, 25], [32, 35, 41], [19, 21, 25], [43, 46, 54], [54, 58, 67], [48, 52, 60], [66, 71, 82], [96, 102, 116], [232, 234, 238]),
    }
}

/// Vignette de choix d'un thème : mini-fenêtre colorée avec le nom dessous.
/// `appear` (0 à 1) : apparition de la carte à l'arrivée sur la page.
pub(crate) fn theme_card(ui: &mut egui::Ui, selected: bool, name: &str, pal: &Pal, acc: Color32, appear: f32) -> egui::Response {
    let (slot, resp) = ui.allocate_exact_size(vec2(152.0, 88.0), Sense::click());
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let ctx = ui.ctx().clone();
    let hk = ctx.animate_bool_with_time_and_easing(resp.id.with("hov"), resp.hovered(), super::anim::HOVER, egui::emath::easing::cubic_out);
    let sk = ctx.animate_bool_with_time_and_easing(resp.id.with("sel"), selected, 0.45, egui::emath::easing::back_out);
    // onde au clic, qui part du point cliqué
    let rip_id = resp.id.with("ripple");
    let now = ctx.input(|i| i.time);
    if resp.clicked() {
        if let Some(pos) = resp.interact_pointer_pos() {
            ctx.data_mut(|d| d.insert_temp(rip_id, (pos, now)));
        }
    }
    // la carte arrive en montant, se soulève au survol
    let rect = Rect::from_center_size(slot.center() + vec2(0.0, 16.0 * (1.0 - appear)), slot.size() * (0.92 + 0.08 * appear));
    let rect = rect.translate(vec2(0.0, -4.0 * hk)).expand(2.0 * hk);
    let mut p = ui.painter().clone();
    p.multiply_opacity(appear);
    let p = &p;
    if hk > 0.01 {
        let sh = egui::epaint::Shadow {
            offset: vec2(0.0, 8.0 * hk),
            blur: 22.0 * hk,
            spread: 0.0,
            color: Color32::from_black_alpha((110.0 * hk) as u8),
        };
        p.add(sh.as_shape(rect, 10.0));
    }
    p.rect_filled(rect, 10.0, pal.bg);
    let side = Rect::from_min_max(rect.min, pos2(rect.min.x + 34.0, rect.max.y));
    p.rect_filled(side, Rounding { nw: 10.0, sw: 10.0, ne: 0.0, se: 0.0 }, pal.side);
    p.rect_filled(Rect::from_min_size(side.min + vec2(8.0, 12.0), vec2(18.0, 5.0)), 2.0, acc);
    p.rect_filled(Rect::from_min_size(side.min + vec2(8.0, 24.0), vec2(18.0, 4.0)), 2.0, pal.line);
    let card = Rect::from_min_max(rect.min + vec2(44.0, 10.0), pos2(rect.max.x - 10.0, rect.min.y + 52.0));
    p.rect_filled(card, 6.0, pal.card);
    // au survol, la mini-interface « s'anime » : le bouton s'allonge, les lignes de texte se remplissent
    let inner_w = card.width() - 16.0;
    p.rect_filled(Rect::from_min_size(card.min + vec2(8.0, 8.0), vec2(34.0 + (inner_w - 34.0) * 0.45 * hk, 8.0)), 4.0, acc);
    p.rect_filled(Rect::from_min_size(card.min + vec2(8.0, 22.0), vec2(inner_w * (0.85 + 0.15 * hk), 4.0)), 2.0, pal.line);
    p.rect_filled(Rect::from_min_size(card.min + vec2(8.0, 31.0), vec2(inner_w * (0.6 + 0.3 * hk), 4.0)), 2.0, pal.line);
    p.text(pos2(rect.min.x + 44.0, rect.max.y - 17.0), Align2::LEFT_CENTER, name, FontId::proportional(13.0), pal.text);
    if let Some((pos, t0)) = ctx.data(|d| d.get_temp::<(Pos2, f64)>(rip_id)) {
        let e = ((now - t0) as f32 / 0.65).clamp(0.0, 1.0);
        if e < 1.0 {
            let r = ease_out(e) * rect.width() * 1.15;
            p.with_clip_rect(rect.intersect(p.clip_rect())).circle_filled(pos, r, acc.gamma_multiply(0.35 * (1.0 - e)));
            ctx.request_repaint();
        }
    }
    // contour : il prend la couleur d'accent et s'épaissit quand la carte devient le thème actif
    let s = sk.clamp(0.0, 1.0);
    let base = pal.line.lerp_to_gamma(pal.stroke_hov, hk);
    p.rect_stroke(rect, 10.0, Stroke::new(1.0 + 1.5 * s, base.lerp_to_gamma(acc, s)));
    if sk > 0.01 {
        // pastille « coché » qui apparaît en rebondissant
        let c = rect.right_top() + vec2(-14.0, 14.0);
        p.circle_filled(c, 9.0 * sk.max(0.0), acc);
        if sk > 0.5 {
            icon_check(p, c, on_accent(acc));
        }
    }
    resp
}

/// Thème sobre : gris neutres + une seule couleur d'accent.
pub(crate) fn apply_style(ctx: &egui::Context, cfg: &Config) {
    apply_style_with(ctx, &palette(&cfg.theme), accent(cfg));
}

/// Mélange de deux palettes (t = 0 : `a`, t = 1 : `b`), pour passer d'un thème à l'autre en douceur.
pub(crate) fn lerp_pal(a: &Pal, b: &Pal, t: f32) -> Pal {
    let m = |x: Color32, y: Color32| x.lerp_to_gamma(y, t);
    Pal {
        dark: if t < 0.5 { a.dark } else { b.dark },
        bg: m(a.bg, b.bg),
        side: m(a.side, b.side),
        card: m(a.card, b.card),
        input: m(a.input, b.input),
        button: m(a.button, b.button),
        hover: m(a.hover, b.hover),
        line: m(a.line, b.line),
        stroke_in: m(a.stroke_in, b.stroke_in),
        stroke_hov: m(a.stroke_hov, b.stroke_hov),
        text: m(a.text, b.text),
    }
}

pub(crate) fn apply_style_with(ctx: &egui::Context, pal: &Pal, acc: Color32) {
    let pal = *pal;
    let (bg, card, input, hover, button, line, text) = (pal.bg, pal.card, pal.input, pal.hover, pal.button, pal.line, pal.text);
    let mut v = if pal.dark { egui::Visuals::dark() } else { egui::Visuals::light() };
    v.override_text_color = Some(text);
    v.panel_fill = bg;
    v.window_fill = card;
    v.extreme_bg_color = input;
    v.faint_bg_color = card;
    v.window_stroke = Stroke::new(1.0_f32, line);
    v.window_rounding = Rounding::same(12.0);
    v.menu_rounding = Rounding::same(8.0);
    v.selection.bg_fill = acc;
    v.selection.stroke = Stroke::new(1.0_f32, on_accent(acc));
    v.hyperlink_color = acc;
    let round = Rounding::same(8.0);
    for (w, fill) in [
        (&mut v.widgets.inactive, button),
        (&mut v.widgets.hovered, hover),
        (&mut v.widgets.active, acc),
        (&mut v.widgets.open, hover),
    ] {
        w.bg_fill = fill;
        w.weak_bg_fill = fill;
        w.bg_stroke = Stroke::NONE;
        w.rounding = round;
        w.expansion = 0.0;
    }
    let (s_in, s_hov) = (pal.stroke_in, pal.stroke_hov);
    v.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, s_in); // cases, radios, champs et boutons visibles
    v.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, s_hov);
    v.slider_trailing_fill = true;
    v.widgets.active.fg_stroke = Stroke::new(1.5_f32, on_accent(acc));
    v.widgets.noninteractive.bg_fill = bg;
    v.widgets.noninteractive.weak_bg_fill = bg;
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, line);
    v.widgets.noninteractive.rounding = round;
    ctx.set_visuals(v);
    ctx.style_mut(|s| {
        s.animation_time = 0.2; // transitions intégrées d'egui (sections repliables, défilement…)
        s.spacing.item_spacing = vec2(10.0, 10.0);
        s.spacing.button_padding = vec2(14.0, 7.0);
        s.spacing.interact_size.y = 30.0;
        s.spacing.slider_width = 180.0;
        s.text_styles.insert(TextStyle::Body, FontId::proportional(15.0));
        s.text_styles.insert(TextStyle::Button, FontId::proportional(15.0));
        s.text_styles.insert(TextStyle::Small, FontId::proportional(12.5));
        s.text_styles.insert(TextStyle::Heading, FontId::new(22.0, egui::FontFamily::Name("bold".into())));
    });
}

pub(crate) fn icon_copy(p: &egui::Painter, c: Pos2, bg: Color32) {
    let st = Stroke::new(1.6_f32, Color32::WHITE);
    p.rect_stroke(Rect::from_center_size(c + vec2(2.5, -2.5), vec2(9.0, 10.0)), 2.0, st);
    let front = Rect::from_center_size(c + vec2(-2.0, 2.0), vec2(9.0, 10.0));
    p.rect_filled(front, 2.0, bg);
    p.rect_stroke(front, 2.0, st);
}

pub(crate) fn icon_trash(p: &egui::Painter, c: Pos2) {
    let st = Stroke::new(1.6_f32, Color32::WHITE);
    p.line_segment([c + vec2(-6.0, -4.5), c + vec2(6.0, -4.5)], st);
    p.line_segment([c + vec2(-2.0, -4.5), c + vec2(-2.0, -7.0)], st);
    p.line_segment([c + vec2(2.0, -4.5), c + vec2(2.0, -7.0)], st);
    p.line_segment([c + vec2(-2.0, -7.0), c + vec2(2.0, -7.0)], st);
    p.rect_stroke(Rect::from_center_size(c + vec2(0.0, 2.5), vec2(9.0, 10.0)), 1.5, st);
}

pub(crate) fn icon_check(p: &egui::Painter, c: Pos2, color: Color32) {
    let st = Stroke::new(2.0_f32, color);
    p.line_segment([c + vec2(-4.5, 0.5), c + vec2(-1.5, 3.5)], st);
    p.line_segment([c + vec2(-1.5, 3.5), c + vec2(4.5, -3.0)], st);
}

/// Petit bouton « panneau latéral » (rectangle avec un trait à gauche), pour ouvrir / réduire le menu.
pub(crate) fn sidebar_icon_button(ui: &mut egui::Ui) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(30.0, 30.0), Sense::click());
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let hover = ui.visuals().widgets.hovered.weak_bg_fill;
    let color = ui.visuals().text_color();
    let hk = ui.ctx().animate_bool_with_time(resp.id.with("hov"), resp.hovered(), super::anim::HOVER);
    let p = ui.painter();
    if hk > 0.01 {
        p.rect_filled(rect, 8.0, hover.gamma_multiply(hk));
    }
    let c = rect.center();
    let st = Stroke::new(1.6_f32, color);
    p.rect_stroke(Rect::from_center_size(c, vec2(17.0, 14.0)), 3.5, st);
    p.line_segment([c + vec2(-3.0, -7.0), c + vec2(-3.0, 7.0)], st);
    resp
}

/// Entrée de la barre latérale : fond teinté + liseré d'accent quand elle est active.
pub(crate) fn nav_item<T: PartialEq + Copy>(ui: &mut egui::Ui, cur: &mut T, tab: T, label: &str) {
    let selected = *cur == tab;
    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 38.0), Sense::click());
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    // survol en fondu ; l'entrée active se teinte, son liseré grandit et le texte se décale un peu
    let hk = ui.ctx().animate_bool_with_time(resp.id.with("hov"), resp.hovered(), super::anim::HOVER);
    let sk = ui.ctx().animate_bool_with_time_and_easing(resp.id.with("sel"), selected, super::anim::PAGE, egui::emath::easing::cubic_out);
    let v = ui.visuals();
    let acc = v.selection.bg_fill;
    let hover = v.widgets.hovered.weak_bg_fill;
    let txt = v.text_color();
    let p = ui.painter();
    if hk * (1.0 - sk) > 0.01 {
        p.rect_filled(rect, 8.0, hover.gamma_multiply(hk * (1.0 - sk)));
    }
    if sk > 0.01 {
        p.rect_filled(rect, 8.0, Color32::from_rgba_unmultiplied(acc.r(), acc.g(), acc.b(), (45.0 * sk) as u8));
        let h = (rect.height() - 18.0) * sk;
        p.rect_filled(Rect::from_center_size(pos2(rect.min.x + 1.5, rect.center().y), vec2(3.0, h)), 2.0, acc);
    }
    let font = if selected {
        FontId::new(15.0, egui::FontFamily::Name("bold".into()))
    } else {
        FontId::proportional(15.0)
    };
    p.text(rect.left_center() + vec2(16.0 + 4.0 * sk, 0.0), Align2::LEFT_CENTER, label, font, txt);
    if resp.clicked() {
        *cur = tab;
    }
}

pub(crate) fn tex(ctx: &egui::Context, name: &str, img: &image::RgbaImage) -> TextureHandle {
    let ci = egui::ColorImage::from_rgba_unmultiplied([img.width() as usize, img.height() as usize], img.as_raw());
    ctx.load_texture(name, ci, Default::default())
}

pub(crate) fn load_thumb(ctx: &egui::Context, path: &Path) -> Option<TextureHandle> {
    let tp = thumb_path(path);
    if !tp.exists() {
        make_thumb(&image::open(path).ok()?.to_rgba8(), &tp); // miniature manquante : on la crée
    }
    Some(tex(ctx, "t", &image::open(&tp).ok()?.to_rgba8()))
}

pub(crate) fn day_label(t: SystemTime) -> String {
    let d = chrono::DateTime::<chrono::Local>::from(t).date_naive();
    let today = chrono::Local::now().date_naive();
    if d == today {
        tr("Aujourd'hui").into()
    } else if Some(d) == today.pred_opt() {
        tr("Hier").into()
    } else {
        d.format(dfmt(true)).to_string()
    }
}


pub(crate) fn file_name(p: &Path) -> String {
    p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()
}

pub(crate) fn hotkeys_ok(c: &Config) -> bool {
    [
        &c.hotkey,
        &c.hotkey_region,
        &c.hotkey_window,
        &c.hotkey_last_region,
        &c.hotkey_gallery,
        &c.hotkey_region_clip,
        &c.hotkey_picker,
    ]
        .iter()
        .all(|s| s.is_empty() || s.parse::<global_hotkey::hotkey::HotKey>().is_ok())
}

pub(crate) fn card(ui: &mut egui::Ui, title: &str, add: impl FnOnce(&mut egui::Ui)) {
    let fill = ui.visuals().faint_bg_color;
    let stroke = ui.visuals().widgets.noninteractive.bg_stroke;
    egui::Frame::none()
        .fill(fill)
        .stroke(stroke)
        .rounding(10.0_f32)
        .inner_margin(egui::Margin::same(16.0))
        .show(ui, |ui| {
            ui.set_width(ui.available_width() - 14.0);
            ui.label(bold_text(title, 17.0));
            ui.add_space(2.0);
            add(ui);
        });
    ui.add_space(12.0);
}

/// En-tête d'une rubrique des paramètres.
pub(crate) fn page_head(ui: &mut egui::Ui, title: &str, sub: &str) {
    ui.label(bold_text(title, 20.0));
    ui.weak(sub);
    ui.add_space(8.0);
}

/// Un réglage : titre, courte explication, puis le contrôle.
pub(crate) fn setting(ui: &mut egui::Ui, title: &str, desc: &str, add: impl FnOnce(&mut egui::Ui)) {
    ui.label(bold_text(title, 15.0));
    if !desc.is_empty() {
        ui.weak(desc);
    }
    ui.add_space(2.0);
    add(ui);
}

pub(crate) fn divider(ui: &mut egui::Ui) {
    ui.add_space(6.0);
    ui.separator();
    ui.add_space(6.0);
}

pub(crate) fn hotkey_row(ui: &mut egui::Ui, label: &str, val: &mut String, rec: &mut Option<usize>, idx: usize) {
    ui.horizontal(|ui| {
        ui.allocate_ui_with_layout(vec2(175.0, 24.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.set_min_size(vec2(175.0, 24.0));
            ui.label(label);
        });
        if *rec == Some(idx) {
            ui.colored_label(Color32::GOLD, tr("Appuie sur ta combinaison…  (Échap = annuler · Retour arrière = vider)"));
            if ui.button(tr("Annuler")).clicked() {
                *rec = None;
            }
        } else {
            ui.add(egui::TextEdit::singleline(val).desired_width(170.0).hint_text(tr("désactivé")));
            if ui.button(tr("Enregistrer|raccourci")).clicked() {
                *rec = Some(idx);
            }
            if !val.is_empty() && val.parse::<global_hotkey::hotkey::HotKey>().is_err() {
                ui.colored_label(Color32::LIGHT_RED, "invalide");
            }
        }
    });
}
