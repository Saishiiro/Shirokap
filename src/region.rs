use crate::common::*;
use crate::i18n::*;
use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Key, Rect, Stroke};
use std::{
    sync::{Arc, Mutex},
    thread::JoinHandle,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    FindWindowW, SetForegroundWindow, SetWindowPos, SWP_NOZORDER, SWP_SHOWWINDOW,
};

struct Region {
    img: image::RgbaImage,
    origin: (i32, i32), // position de l'écran capturé sur le bureau
    placed: bool,
    tex: egui::TextureHandle,
    cfg: Config,
    title: String,
    start: Option<egui::Pos2>,
    cur: Option<egui::Pos2>,
    jobs: Arc<Mutex<Vec<JoinHandle<()>>>>,
    clip: bool, // true = vers le presse-papiers, sans fichier
}

/// Place la fenêtre exactement sur l'écran voulu (en pixels réels), quel que soit l'écran ou son échelle.
fn place_window(x: i32, y: i32, w: i32, h: i32) {
    let t: Vec<u16> = "Shirokap zone".encode_utf16().chain([0]).collect();
    unsafe {
        let hwnd = FindWindowW(std::ptr::null(), t.as_ptr());
        if !hwnd.is_null() {
            SetWindowPos(hwnd, std::ptr::null_mut(), x, y, w, h, SWP_NOZORDER | SWP_SHOWWINDOW);
            SetForegroundWindow(hwnd);
        }
    }
}

/// Overlay plein écran sur l'écran où se trouve le curseur : l'écran est figé, on trace un rectangle.
pub fn run(title: String, clip: bool) {
    let Some((img, mx, my)) = monitor_shot_at_cursor() else { return }; // capture AVANT d'afficher la fenêtre
    let cfg = Config::load();
    let jobs: Arc<Mutex<Vec<JoinHandle<()>>>> = Arc::new(Mutex::new(Vec::new()));
    let j2 = jobs.clone();
    let opts = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([200.0, 200.0])
            .with_decorations(false)
            .with_always_on_top()
            .with_title("Shirokap zone"),
        ..Default::default()
    };
    let _ = eframe::run_native(
        "Shirokap zone",
        opts,
        Box::new(move |cc| {
            crate::gallery::ui_kit::install_fonts(&cc.egui_ctx); // Inter + police chinoise si besoin
            let ci = egui::ColorImage::from_rgba_unmultiplied(
                [img.width() as usize, img.height() as usize],
                img.as_raw(),
            );
            let tex = cc.egui_ctx.load_texture("shot", ci, Default::default());
            Ok(Box::new(Region {
                img,
                origin: (mx, my),
                placed: false,
                tex,
                cfg,
                title,
                start: None,
                cur: None,
                jobs: j2,
                clip,
            }))
        }),
    );
    // la fenêtre est fermée : on attend la fin de l'enregistrement en arrière-plan
    let handles: Vec<JoinHandle<()>> = jobs.lock().unwrap().drain(..).collect();
    for h in handles {
        let _ = h.join();
    }
}

impl Region {
    fn save(&mut self, sel: Rect, full: Rect) {
        let (iw, ih) = (self.img.width(), self.img.height());
        let (sx, sy) = (iw as f32 / full.width(), ih as f32 / full.height());
        let x = (((sel.min.x - full.min.x) * sx).round().max(0.0) as u32).min(iw - 1);
        let y = (((sel.min.y - full.min.y) * sy).round().max(0.0) as u32).min(ih - 1);
        let w = ((sel.width() * sx).round() as u32).clamp(1, iw - x);
        let h = ((sel.height() * sy).round() as u32).clamp(1, ih - y);
        let crop = image::imageops::crop_imm(&self.img, x, y, w, h).to_image();
        save_last_region(x, y, w, h, self.origin.0, self.origin.1);
        let (cfg, title, clip) = (self.cfg.clone(), self.title.clone(), self.clip);
        let handle = std::thread::spawn(move || {
            if clip {
                copy_to_clipboard(&crop);
            } else {
                finish(&cfg, crop, &title);
            }
        });
        self.jobs.lock().unwrap().push(handle);
    }
}

impl eframe::App for Region {
    fn update(&mut self, ctx: &egui::Context, _f: &mut eframe::Frame) {
        crate::common::frame_cap(); // au plus 60 images par seconde
        if !self.placed {
            self.placed = true;
            place_window(self.origin.0, self.origin.1, self.img.width() as i32, self.img.height() as i32);
            ctx.request_repaint();
        }
        ctx.set_cursor_icon(egui::CursorIcon::Crosshair);
        let (pressed, down, released, pos, esc, cancel) = ctx.input(|i| {
            (
                i.pointer.primary_pressed(),
                i.pointer.primary_down(),
                i.pointer.primary_released(),
                i.pointer.interact_pos(),
                i.key_pressed(Key::Escape),
                i.pointer.secondary_clicked(),
            )
        });
        if esc || cancel {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        let full = ctx.screen_rect();
        if pressed {
            self.start = pos;
            self.cur = pos;
        } else if down {
            self.cur = pos.or(self.cur);
        }
        if released {
            if let (Some(a), Some(b)) = (self.start, self.cur) {
                let sel = Rect::from_min_max(a.min(b), a.max(b));
                if sel.width() > 6.0 && sel.height() > 6.0 {
                    self.save(sel, full);
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    return;
                }
            }
            self.start = None;
            self.cur = None;
        }

        egui::CentralPanel::default().frame(egui::Frame::none()).show(ctx, |ui| {
            let p = ui.painter();
            p.image(self.tex.id(), full, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
            let dim = Color32::from_black_alpha(130);
            match (self.start, self.cur) {
                (Some(a), Some(b)) => {
                    let s = Rect::from_min_max(a.min(b), a.max(b));
                    p.rect_filled(Rect::from_min_max(full.min, pos2(full.max.x, s.min.y)), 0.0, dim);
                    p.rect_filled(Rect::from_min_max(pos2(full.min.x, s.max.y), full.max), 0.0, dim);
                    p.rect_filled(Rect::from_min_max(pos2(full.min.x, s.min.y), pos2(s.min.x, s.max.y)), 0.0, dim);
                    p.rect_filled(Rect::from_min_max(pos2(s.max.x, s.min.y), pos2(full.max.x, s.max.y)), 0.0, dim);
                    p.rect_stroke(s, 0.0, Stroke::new(1.5_f32, Color32::WHITE));
                    let w = (s.width() * self.img.width() as f32 / full.width()).round();
                    let h = (s.height() * self.img.height() as f32 / full.height()).round();
                    let (at, al) = if s.min.y > 26.0 {
                        (s.min - vec2(0.0, 4.0), Align2::LEFT_BOTTOM)
                    } else {
                        (s.min + vec2(6.0, 6.0), Align2::LEFT_TOP)
                    };
                    p.text(at, al, format!("{w} × {h}"), FontId::proportional(15.0), Color32::WHITE);
                }
                _ => {
                    p.rect_filled(full, 0.0, dim);
                    p.text(
                        full.center_top() + vec2(0.0, 48.0),
                        Align2::CENTER_TOP,
                        if self.clip {
                            tr("Zone vers le presse-papiers  ·  Échap ou clic droit pour annuler")
                        } else {
                            tr("Glisse pour sélectionner  ·  Échap ou clic droit pour annuler")
                        },
                        FontId::proportional(22.0),
                        Color32::WHITE,
                    );
                }
            }
        });
    }
}
