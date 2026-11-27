//! Menu « moderne » de l'icône du tray : une petite fenêtre dessinée aux couleurs de l'appli
//! (police Inter, coins arrondis, thème). La commande choisie est déposée dans un fichier que le démon lit.
use crate::common::*;
use crate::i18n::*;
use crate::gallery::ease_out;
use crate::gallery::ui_kit::{accent, apply_style, install_fonts, palette, Pal};
use crate::picker::on_accent;
use crate::icons;
use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Rect, Sense, Stroke, TextureHandle};
use std::time::{Duration, Instant};
use windows_sys::Win32::{
    Foundation::{POINT, RECT},
    Graphics::Gdi::{GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTONEAREST},
    UI::WindowsAndMessaging::{FindWindowW, SetForegroundWindow, SetWindowPos, SWP_NOZORDER, SWP_SHOWWINDOW},
};

const MARGIN: f32 = 14.0; // espace transparent autour des cartes (ombre)
const CARD_W: f32 = 290.0; // largeurs minimales ; elles s'élargissent selon les textes (langue, raccourcis)
const FLY_W: f32 = 300.0; // largeur du sous-menu
const GAP: f32 = 8.0;
const PAD: f32 = 8.0;
const ROW_H: f32 = 36.0;
const SEP_H: f32 = 13.0;
const HEAD_H: f32 = 54.0;
const TILES_H: f32 = 72.0;
const TILE_GAP: f32 = 6.0;

enum It {
    /// en-tête : logo, nom et version, captures du jour (et « En pause » si les raccourcis sont suspendus)
    Header { sub: String, paused: bool },
    /// rangée de grosses tuiles pour les actions les plus courantes
    Tiles(Vec<(String, &'static str, String)>),
    Sep,
    Act { id: String, icon: &'static str, label: String, hint: String, checked: Option<bool> },
    Sub { icon: &'static str, label: String, kids: Vec<It> },
}

fn item_h(it: &It) -> f32 {
    match it {
        It::Header { .. } => HEAD_H,
        It::Tiles(_) => TILES_H,
        It::Sep => SEP_H,
        _ => ROW_H,
    }
}

/// Option à cocher parmi plusieurs (résolution) : une coche à gauche ; les autres options cochables
/// sont des interrupteurs à droite.
fn is_radio(id: &str) -> bool {
    id.starts_with("res:")
}

/// Largeur dont une carte a besoin pour afficher tous ses libellés en entier (mesurée avec les vraies polices).
fn card_need_w(ctx: &egui::Context, items: &[It]) -> f32 {
    let gutter = gutter_of(items);
    let w = |t: &str, size: f32| ctx.fonts(|f| f.layout_no_wrap(t.to_owned(), FontId::proportional(size), Color32::WHITE).size().x);
    let row = items
        .iter()
        .map(|it| match it {
            It::Sep => 0.0,
            It::Header { sub, .. } => 52.0 + w("Shirokap 0.00.0", 15.0).max(w(sub, 12.5)) + 100.0,
            It::Tiles(t) => t.len() as f32 * 60.0 + (t.len().saturating_sub(1)) as f32 * TILE_GAP,
            It::Act { id, label, hint, checked, .. } => {
                let right = if !hint.is_empty() {
                    w(hint, 12.0) + 16.0 + 14.0
                } else if checked.is_some() && !is_radio(id) {
                    48.0
                } else if checked.is_some() {
                    34.0
                } else {
                    12.0
                };
                gutter + w(label, 14.5) + right
            }
            It::Sub { label, .. } => gutter + w(label, 14.5) + 30.0,
        })
        .fold(0.0_f32, f32::max);
    (row + 2.0 * PAD + 6.0).ceil()
}

fn card_h(items: &[It]) -> f32 {
    items.iter().map(item_h).sum::<f32>() + 2.0 * PAD
}

/// Même contenu et mêmes commandes que le menu natif.
fn model(cfg: &Config, paused: bool, auto: bool) -> Vec<It> {
    let act = |id: &str, icon: &'static str, label: &str, hint: &str| It::Act {
        id: id.to_string(),
        icon,
        label: label.to_string(),
        hint: hint.to_string(),
        checked: None,
    };
    let chk = |id: &str, icon: &'static str, label: &str, on: bool| It::Act {
        id: id.to_string(),
        icon,
        label: label.to_string(),
        hint: String::new(),
        checked: Some(on),
    };
    let today = today_label(today_count(cfg));
    let tile = |id: &str, icon: &'static str, label: &str| (id.to_string(), icon, label.to_string());
    let mut quick = vec![
        chk("tog_clip", icons::CLIPBOARD, tr("Copier dans le presse-papiers"), cfg.clipboard),
        chk("tog_sound", icons::SOUND, tr("Bip sonore"), cfg.sound),
        chk("tog_flash", icons::FLASH, tr("Clignotement de l'icône"), cfg.flash),
        chk("tog_folder", icons::FOLDER_TREE, tr("Un dossier par fenêtre / jeu"), cfg.folder_per_window),
        It::Sep,
    ];
    for (w, l) in [
        (0u32, tr("Résolution native")),
        (3840, "4K · 3840 px"),
        (2560, "1440p · 2560 px"),
        (1920, "1080p · 1920 px"),
        (1280, "720p · 1280 px"),
    ] {
        quick.push(chk(&format!("res:{w}"), icons::MONITOR, l, cfg.target_width == w));
    }
    vec![
        It::Header { sub: today, paused },
        It::Tiles(vec![
            tile("snap", icons::CAMERA, tr("Écran")),
            tile("zone", icons::ZONE, tr("Zone")),
            tile("gallery", icons::GALLERY, tr("Galerie")),
            tile("pick", icons::PIPETTE, tr("Pipette")),
        ]),
        It::Sep,
        It::Sub {
            icon: icons::CAMERA,
            label: tr("Capturer").to_string(),
            kids: vec![
                act("snap", icons::SYSTEM, tr("Écran entier"), &cfg.hotkey),
                act("zone", icons::ZONE, tr("Zone…"), &cfg.hotkey_region),
                act("zone_clip", icons::CLIPBOARD, tr("Zone → presse-papiers"), &cfg.hotkey_region_clip),
                act("snap_last", icons::HISTORY, tr("Refaire la dernière zone"), &cfg.hotkey_last_region),
                It::Sep,
                act("delayed_screen", icons::TIMER, tr("Écran dans 5 secondes"), ""),
                act("delayed_window", icons::APP_WINDOW, tr("Fenêtre active dans 5 secondes"), ""),
                It::Sep,
                act("pick", icons::PIPETTE, tr("Pipette : couleur sous le curseur (3 s)"), &cfg.hotkey_picker),
            ],
        },
        chk("auto", icons::CLOCK, tr("Capture automatique (timelapse)"), auto),
        It::Sep,
        act("gallery", icons::GALLERY, tr("Ouvrir la galerie"), &cfg.hotkey_gallery),
        It::Sub {
            icon: icons::IMAGE,
            label: tr("Dernière capture").to_string(),
            kids: vec![
                act("last", icons::EXTERNAL, tr("Ouvrir"), ""),
                act("copy_last", icons::COPY, tr("Copier dans le presse-papiers"), ""),
                act("pin_last", icons::PIN, tr("Épingler à l'écran"), ""),
                It::Sep,
                act("trash_last", icons::TRASH, tr("Mettre à la corbeille"), ""),
            ],
        },
        It::Sub {
            icon: icons::FOLDER,
            label: tr("Dossiers").to_string(),
            kids: vec![
                act("folder", icons::FOLDER_OPEN, tr("Dossier des captures"), ""),
                act("config_dir", icons::FOLDER_COG, tr("Dossier de configuration"), ""),
            ],
        },
        It::Sep,
        It::Sub { icon: icons::SLIDERS, label: tr("Réglages rapides").to_string(), kids: quick },
        chk("pause", icons::PAUSE, tr("Suspendre les raccourcis"), paused),
        act("settings", icons::SETTINGS, tr("Paramètres…"), ""),
        It::Sep,
        act("quit", icons::POWER, tr("Quitter"), ""),
    ]
}

#[derive(Default)]
struct Out {
    chosen: Option<String>,
    hover_sub: Option<usize>,
    hover_plain: bool,
}

/// Texte sur une seule ligne, coupé avec « … » s'il dépasse `max_w` (il ne chevauche jamais le raccourci).
fn fit(p: &egui::Painter, text: &str, font: FontId, color: Color32, max_w: f32) -> std::sync::Arc<egui::Galley> {
    let mut job = egui::text::LayoutJob::simple_singleline(text.to_owned(), font, color);
    job.wrap = egui::text::TextWrapping { max_width: max_w.max(20.0), max_rows: 1, break_anywhere: true, overflow_character: Some('…') };
    p.layout_job(job)
}

/// Place laissée à gauche des libellés : icône, coche, ou rien.
fn gutter_of(items: &[It]) -> f32 {
    let has_radio = items.iter().any(|i| matches!(i, It::Act { id, checked: Some(_), .. } if is_radio(id)));
    let has_icon = items.iter().any(|i| matches!(i, It::Act { icon, .. } | It::Sub { icon, .. } if !icon.is_empty()));
    if has_icon {
        44.0
    } else if has_radio {
        34.0
    } else {
        14.0
    }
}

/// Petit interrupteur (option activée / désactivée), centré sur `c`.
fn switch(p: &egui::Painter, c: egui::Pos2, on: bool, pal: &Pal, acc: Color32) {
    let track = Rect::from_center_size(c, vec2(32.0, 18.0));
    p.rect_filled(track, 9.0, if on { acc } else { pal.button });
    if !on {
        p.rect_stroke(track, 9.0, Stroke::new(1.0_f32, pal.stroke_in));
    }
    let x = if on { track.max.x - 9.0 } else { track.min.x + 9.0 };
    let knob = if on { on_accent(acc) } else { pal.text.gamma_multiply(0.75) };
    p.circle_filled(pos2(x, c.y), 6.0, knob);
}

/// Couleur des icônes des tuiles : la couleur d'accent, sauf si elle se voit mal sur le fond.
fn readable(acc: Color32, bg: Color32, text: Color32) -> Color32 {
    let l = |c: Color32| 0.299 * c.r() as f32 + 0.587 * c.g() as f32 + 0.114 * c.b() as f32;
    if (l(acc) - l(bg)).abs() < 70.0 {
        text
    } else {
        acc
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_card(ui: &mut egui::Ui, pal: &Pal, acc: Color32, logo: Option<&TextureHandle>, rect: Rect, items: &[It], salt: &str, active_sub: Option<usize>, alpha: f32, out: &mut Out) {
    let mut p = ui.painter().clone();
    p.multiply_opacity(alpha);
    let p = &p;
    let shadow = egui::epaint::Shadow { offset: vec2(0.0, 8.0), blur: 24.0, spread: 0.0, color: Color32::from_black_alpha(if pal.dark { 120 } else { 50 }) };
    p.add(shadow.as_shape(rect, 16.0));
    p.rect_filled(rect, 16.0, pal.bg);
    p.rect_stroke(rect, 16.0, Stroke::new(1.0_f32, pal.line));
    let gutter = gutter_of(items);
    let icon_font = FontId::proportional(16.0);
    let bold = |size: f32| FontId::new(size, egui::FontFamily::Name("bold".into()));
    let t = pal.text;
    let weak = Color32::from_rgba_unmultiplied(t.r(), t.g(), t.b(), 140);
    let mut y = rect.min.y + PAD;
    for (n, it) in items.iter().enumerate() {
        let h = item_h(it);
        let row = Rect::from_min_size(pos2(rect.min.x + PAD, y), vec2(rect.width() - 2.0 * PAD, h));
        match it {
            It::Sep => {
                let ly = row.center().y;
                p.line_segment([pos2(row.min.x + 10.0, ly), pos2(row.max.x - 10.0, ly)], Stroke::new(1.0_f32, pal.line));
            }
            It::Header { sub, paused } => {
                let logo_r = Rect::from_center_size(pos2(row.min.x + 22.0, row.center().y), vec2(32.0, 32.0));
                if let Some(tex) = logo {
                    p.image(tex.id(), logo_r, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
                }
                let x = logo_r.max.x + 11.0;
                let name = p.text(pos2(x, row.center().y - 9.0), Align2::LEFT_CENTER, "Shirokap", bold(15.0), t);
                p.text(pos2(name.max.x + 6.0, row.center().y - 8.5), Align2::LEFT_CENTER, env!("CARGO_PKG_VERSION"), FontId::proportional(12.0), weak);
                p.text(pos2(x, row.center().y + 10.0), Align2::LEFT_CENTER, sub, FontId::proportional(12.5), weak);
                if *paused {
                    // « En pause » : petite pastille ambrée à droite
                    let amber = Color32::from_rgb(251, 191, 36);
                    let g = p.layout_no_wrap(tr("En pause").to_owned(), bold(11.5), amber);
                    let chip = Rect::from_min_size(pos2(row.max.x - 10.0 - g.size().x - 20.0, row.center().y - 11.0), vec2(g.size().x + 20.0, 22.0));
                    p.rect_filled(chip, 11.0, Color32::from_rgba_unmultiplied(251, 191, 36, 40));
                    p.galley(chip.center() - g.size() / 2.0, g, amber);
                }
            }
            It::Tiles(tiles) => {
                let nt = tiles.len().max(1) as f32;
                let tw = (row.width() - (nt - 1.0) * TILE_GAP) / nt;
                for (k, (id, icon, label)) in tiles.iter().enumerate() {
                    let r = Rect::from_min_size(pos2(row.min.x + k as f32 * (tw + TILE_GAP), row.min.y + 4.0), vec2(tw, h - 8.0));
                    let resp = ui.interact(r, egui::Id::new((salt, n, k)), Sense::click());
                    let hk = ui.ctx().animate_bool_with_time(resp.id.with("hov"), resp.hovered(), 0.12);
                    if resp.hovered() {
                        out.hover_plain = true;
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    p.rect_filled(r, 12.0, pal.card.lerp_to_gamma(pal.hover, hk));
                    p.text(pos2(r.center().x, r.min.y + 22.0), Align2::CENTER_CENTER, *icon, FontId::proportional(19.0), readable(acc, pal.card, t));
                    let g = fit(p, label, FontId::proportional(12.5), t, tw - 8.0);
                    p.galley(pos2(r.center().x - g.size().x / 2.0, r.max.y - 12.0 - g.size().y / 2.0), g, t);
                    if resp.clicked() {
                        out.chosen = Some(id.clone());
                    }
                }
            }
            It::Act { id, icon, label, hint, checked } => {
                let resp = ui.interact(row, egui::Id::new((salt, n)), Sense::click());
                let hk = ui.ctx().animate_bool_with_time(resp.id.with("hov"), resp.hovered(), 0.12);
                if hk > 0.01 {
                    p.rect_filled(row, 10.0, pal.hover.gamma_multiply(hk));
                }
                if resp.hovered() {
                    out.hover_plain = true;
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                let on = *checked == Some(true);
                let radio = is_radio(id);
                if !icon.is_empty() {
                    p.text(pos2(row.min.x + 15.0, row.center().y), Align2::LEFT_CENTER, *icon, icon_font.clone(), if on { readable(acc, pal.bg, t) } else { t });
                    if on && radio {
                        // option choisie parmi plusieurs : coche à droite
                        let c = pos2(row.max.x - 17.0, row.center().y);
                        let st = Stroke::new(2.0_f32, readable(acc, pal.bg, t));
                        p.line_segment([c + vec2(-4.5, 0.5), c + vec2(-1.5, 3.5)], st);
                        p.line_segment([c + vec2(-1.5, 3.5), c + vec2(4.5, -3.5)], st);
                    }
                } else if on && radio {
                    let c = pos2(row.min.x + 17.0, row.center().y);
                    let st = Stroke::new(2.0_f32, readable(acc, pal.bg, t));
                    p.line_segment([c + vec2(-4.5, 0.5), c + vec2(-1.5, 3.5)], st);
                    p.line_segment([c + vec2(-1.5, 3.5), c + vec2(4.5, -3.5)], st);
                }
                // place laissée au libellé : jusqu'au raccourci (ou à l'interrupteur) à droite
                let mut right = row.max.x - 12.0;
                if !hint.is_empty() {
                    // raccourci dans une petite étiquette
                    let g = p.layout_no_wrap(hint.clone(), FontId::proportional(11.5), weak);
                    let chip = Rect::from_min_size(pos2(row.max.x - 10.0 - g.size().x - 14.0, row.center().y - 10.0), vec2(g.size().x + 14.0, 20.0));
                    p.rect_filled(chip, 10.0, pal.button);
                    p.galley(chip.center() - g.size() / 2.0, g, weak);
                    right = chip.min.x - 12.0;
                } else if checked.is_some() && !radio {
                    switch(p, pos2(row.max.x - 26.0, row.center().y), on, pal, acc);
                    right = row.max.x - 52.0;
                } else if radio && !icon.is_empty() {
                    right = row.max.x - 34.0;
                }
                let g = fit(p, label, FontId::proportional(14.5), t, right - (row.min.x + gutter));
                p.galley(pos2(row.min.x + gutter, row.center().y - g.size().y / 2.0), g, t);
                if resp.clicked() {
                    out.chosen = Some(id.clone());
                }
            }
            It::Sub { icon, label, .. } => {
                let resp = ui.interact(row, egui::Id::new((salt, n)), Sense::click());
                let open = resp.hovered() || active_sub == Some(n);
                let hk = ui.ctx().animate_bool_with_time(resp.id.with("hov"), open, 0.12);
                if hk > 0.01 {
                    p.rect_filled(row, 10.0, pal.hover.gamma_multiply(hk));
                }
                if resp.hovered() {
                    out.hover_sub = Some(n);
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                if !icon.is_empty() {
                    p.text(pos2(row.min.x + 15.0, row.center().y), Align2::LEFT_CENTER, *icon, icon_font.clone(), t);
                }
                let g = fit(p, label, FontId::proportional(14.5), t, row.max.x - 30.0 - (row.min.x + gutter));
                p.galley(pos2(row.min.x + gutter, row.center().y - g.size().y / 2.0), g, t);
                let c = pos2(row.max.x - 16.0, row.center().y);
                let st = Stroke::new(1.7_f32, weak);
                p.line_segment([c + vec2(-2.5, -4.5), c + vec2(2.0, 0.0)], st);
                p.line_segment([c + vec2(2.0, 0.0), c + vec2(-2.5, 4.5)], st);
            }
        }
        y += h;
    }
}

struct Popup {
    items: Vec<It>,
    pal: Pal,
    acc: Color32,
    pos: (i32, i32),
    win: (f32, f32),
    card_w: f32,
    fly_w: f32,
    placed: bool,
    start: Instant,
    had_focus: bool,
    open_sub: Option<usize>,
    sub_at: Instant, // ouverture du sous-menu (il apparaît en fondu)
    fly_rect: Option<Rect>,
    logo: Option<TextureHandle>,
}

pub fn run(x: i32, y: i32, paused: bool, auto: bool) {
    let cfg = Config::load();
    let items = model(&cfg, paused, auto);
    let main_h = card_h(&items);
    let fly_h = items
        .iter()
        .filter_map(|i| if let It::Sub { kids, .. } = i { Some(card_h(kids)) } else { None })
        .fold(0.0_f32, f32::max);
    let win = (MARGIN * 2.0 + FLY_W + GAP + CARD_W, MARGIN * 2.0 + main_h.max(fly_h));
    let opts = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([win.0, win.1])
            .with_decorations(false)
            .with_transparent(true)
            .with_always_on_top()
            .with_taskbar(false)
            .with_title("Shirokap menu"),
        ..Default::default()
    };
    let _ = eframe::run_native(
        "Shirokap menu",
        opts,
        Box::new(move |cc| {
            install_fonts(&cc.egui_ctx);
            apply_style(&cc.egui_ctx, &cfg);
            let logo = image::load_from_memory(include_bytes!("../assets/logo.png")).ok().map(|i| {
                let i = i.resize_exact(64, 64, image::imageops::FilterType::Lanczos3).to_rgba8();
                let ci = egui::ColorImage::from_rgba_unmultiplied([64, 64], i.as_raw());
                cc.egui_ctx.load_texture("logo", ci, egui::TextureOptions::LINEAR)
            });
            Ok(Box::new(Popup {
                items,
                pal: palette(&cfg.theme),
                acc: accent(&cfg),
                pos: (x, y),
                win,
                card_w: CARD_W,
                fly_w: FLY_W,
                placed: false,
                start: Instant::now(),
                had_focus: false,
                open_sub: None,
                sub_at: Instant::now(),
                fly_rect: None,
                logo,
            }))
        }),
    );
}

impl Popup {
    /// Place la fenêtre près de l'icône (au-dessus de la barre des tâches), dans la zone utile de l'écran.
    fn place(&self, ctx: &egui::Context) {
        let ppp = ctx.pixels_per_point();
        let (w, h) = ((self.win.0 * ppp) as i32, (self.win.1 * ppp) as i32);
        let m = (MARGIN * ppp) as i32;
        let (cx, cy) = self.pos;
        let zero = RECT { left: 0, top: 0, right: 0, bottom: 0 };
        let mut mi = MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, rcMonitor: zero, rcWork: zero, dwFlags: 0 };
        unsafe {
            let hm = MonitorFromPoint(POINT { x: cx, y: cy }, MONITOR_DEFAULTTONEAREST);
            GetMonitorInfoW(hm, &mut mi);
        }
        let work = mi.rcWork;
        let px = |v: f32| (v * ppp) as i32;
        // bord droit de la carte principale : à droite de l'icône, sans dépasser de l'écran
        let card_right = (cx + px(24.0)).min(work.right - px(8.0));
        let x = (card_right + m - w).max(work.left - m);
        let y = if cy > (work.top + work.bottom) / 2 {
            let card_bottom = (cy - px(4.0)).min(work.bottom - px(8.0));
            card_bottom + m - h
        } else {
            let card_top = (cy + px(8.0)).max(work.top + px(8.0));
            card_top - m
        };
        let y = y.max(work.top - m);
        let title: Vec<u16> = "Shirokap menu".encode_utf16().chain([0]).collect();
        unsafe {
            let hwnd = FindWindowW(std::ptr::null(), title.as_ptr());
            if !hwnd.is_null() {
                SetWindowPos(hwnd, std::ptr::null_mut(), x, y, w, h, SWP_NOZORDER | SWP_SHOWWINDOW);
                SetForegroundWindow(hwnd);
            }
        }
    }
}

impl eframe::App for Popup {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0] // fenêtre transparente : seules les cartes arrondies sont visibles
    }

    fn update(&mut self, ctx: &egui::Context, _f: &mut eframe::Frame) {
        if !self.placed {
            self.placed = true;
            // largeurs des cartes selon leurs textes : la fenêtre prend la taille voulue au moment d'être placée
            self.card_w = card_need_w(ctx, &self.items).max(CARD_W);
            self.fly_w = self
                .items
                .iter()
                .filter_map(|i| if let It::Sub { kids, .. } = i { Some(card_need_w(ctx, kids)) } else { None })
                .fold(FLY_W, f32::max);
            self.win.0 = MARGIN * 2.0 + self.fly_w + GAP + self.card_w;
            self.place(ctx);
            ctx.request_repaint();
            return; // rien n'est dessiné avant le placement (pas de clignotement)
        }
        let (esc, focused, click, pos) = ctx.input(|i| {
            (i.key_pressed(egui::Key::Escape), i.focused, i.pointer.any_click(), i.pointer.interact_pos())
        });
        if focused {
            self.had_focus = true;
        }
        let age = self.start.elapsed().as_millis();
        // le menu apparaît en fondu, le sous-menu aussi à chaque ouverture
        let k = ease_out((self.start.elapsed().as_secs_f32() / 0.14).min(1.0));
        let sk = ease_out((self.sub_at.elapsed().as_secs_f32() / 0.12).min(1.0));
        if k < 1.0 || sk < 1.0 {
            ctx.request_repaint();
        }
        // fermeture : Échap, clic ailleurs (perte du focus), ou jamais activé
        if esc || (self.had_focus && !focused && age > 250) || (!self.had_focus && age > 2500) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        ctx.request_repaint_after(Duration::from_millis(100));

        egui::CentralPanel::default().frame(egui::Frame::none()).show(ctx, |ui| {
            let full = ui.max_rect();
            let main_h = card_h(&self.items);
            let main = Rect::from_min_size(pos2(full.max.x - MARGIN - self.card_w, full.max.y - MARGIN - main_h), vec2(self.card_w, main_h));
            let hover = ctx.input(|i| i.pointer.hover_pos());
            let mut out = Out::default();
            draw_card(ui, &self.pal, self.acc, self.logo.as_ref(), main, &self.items, "main", self.open_sub, k, &mut out);

            // sous-menu : ouvert au survol d'une ligne à flèche, conservé tant que la souris est dessus
            let over_fly = match (self.fly_rect, hover) {
                (Some(r), Some(p)) => r.contains(p),
                _ => false,
            };
            if let Some(n) = out.hover_sub {
                if self.open_sub != Some(n) {
                    self.sub_at = Instant::now();
                }
                self.open_sub = Some(n);
            } else if out.hover_plain && !over_fly {
                self.open_sub = None;
            }
            self.fly_rect = None;
            if let Some(n) = self.open_sub {
                if let Some(It::Sub { kids, .. }) = self.items.get(n) {
                    let row_y = main.min.y + PAD + self.items[..n].iter().map(item_h).sum::<f32>();
                    let fh = card_h(kids);
                    let top = (row_y - PAD).clamp(full.min.y + MARGIN, (full.max.y - MARGIN - fh).max(full.min.y + MARGIN));
                    let fr = Rect::from_min_size(pos2(main.min.x - GAP - self.fly_w, top), vec2(self.fly_w, fh));
                    draw_card(ui, &self.pal, self.acc, None, fr, kids, &format!("fly{n}"), None, k.min(sk), &mut out);
                    self.fly_rect = Some(fr);
                }
            }

            if let Some(id) = out.chosen.take() {
                let _ = std::fs::write(popup_cmd_path(), id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            } else if click {
                if let Some(p) = pos {
                    let inside = main.contains(p) || self.fly_rect.map(|r| r.contains(p)).unwrap_or(false);
                    if !inside {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                }
            }
        });
    }
}
