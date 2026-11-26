use crate::common::*;
use crate::i18n::*;
use crate::editor::{self, dropdown, Editor};
use crate::picker::{color_button, on_accent};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
use eframe::egui::{
    self, pos2, vec2, Align2, Color32, FontId, Key, Pos2, Rect, Rounding, Sense, Stroke, TextStyle, TextureHandle,
};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant, SystemTime},
};

mod grid;
mod palette;
mod hotkeys;
mod settings;
pub(crate) mod ui_kit;
mod views;

use hotkeys::*;
use ui_kit::*;
use crate::icons;


#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Gallery,
    Settings,
    Stats,
}
#[derive(Clone, Copy, PartialEq)]
enum SetCat {
    Shortcuts,
    Capture,
    Files,
    Look,
    Privacy,
    System,
}

#[derive(Clone, Copy, PartialEq)]
enum Sort {
    Newest,
    Oldest,
    Name,
    Size,
}
enum Act {
    Open(usize),
    Pick(PathBuf),
    Fav(PathBuf),
    Copy(PathBuf),
    Reveal(PathBuf),
    Trash(Vec<PathBuf>),
    Path(PathBuf),
    Edit(PathBuf),
    Pin(PathBuf),
    Annotate(PathBuf),
    CopyFile(PathBuf),
    Compare,
    CopyFiles,
    Export,
    Rename(PathBuf),
}

/// Comparateur à curseur (deux images superposées).
struct Cmp {
    a: TextureHandle,
    b: TextureHandle,
    na: String,
    nb: String,
    split: f32,
}

struct App {
    cfg: Config,
    saved: String,
    applied: (String, [u8; 3]),
    shown_pal: Pal,      // palette affichée (mélange pendant un changement de thème)
    shown_acc: Color32,  // couleur d'accent affichée
    theme_anim: Option<(Pal, Color32, Instant)>, // départ d'un changement d'accent : couleurs d'origine + moment
    hwnd: isize,         // fenêtre Windows de la galerie (pour la barre de titre)
    look_key: Option<(bool, [u8; 3])>, // (thème sombre, fond) déjà appliqués à la barre de titre
    cmd: Option<palette::CmdPalette>, // palette de commandes ouverte (Ctrl+K)
    vis_cache: Option<(u64, std::rc::Rc<VisGroups>)>, // captures affichées et leurs groupes par jour, recalculés seulement si besoin
    thumb_rects: HashMap<usize, Rect>, // position des miniatures affichées (pour l'image qui vole)
    fly_from: Option<Rect>, // miniature d'où part la capture qu'on ouvre
    fly: Option<Fly>,    // image qui vole entre sa miniature et la visionneuse
    view_rect: Option<Rect>, // dernière position de l'image dans la visionneuse
    reveal_req: Option<(Pos2, Instant)>, // changement de thème : capture de l'ancien thème demandée (origine du cercle, moment)
    reveal: Option<(TextureHandle, Pos2, Instant)>, // ancien thème en image, qu'un cercle grandissant découvre
    applied_lang: String,
    fonts_cjk: bool, // police chinoise actuellement chargée
    tab: Tab,
    items: Vec<Item>,
    favs: HashSet<PathBuf>,
    sel: HashSet<PathBuf>,
    folder: String,
    search: String,
    sort: Sort,
    favs_only: bool,
    thumb_w: f32,
    thumbs: HashMap<PathBuf, Option<TextureHandle>>,
    thumb_born: HashMap<PathBuf, Instant>, // moment du chargement de chaque miniature (pour son fondu)
    view: Option<usize>,
    full: Option<(PathBuf, TextureHandle)>,
    zoom: crate::zoom::Zoom,
    autostart: bool,
    focused: bool,
    status: String,
    stat: (usize, u64),
    rename: Option<(PathBuf, String)>,
    slideshow: bool,
    slide_at: Instant,
    slide_new_first: bool,
    fullscreen: bool,
    editor: Option<Editor>,
    compare: Option<Cmp>,
    recording: Option<usize>, // raccourci en cours d'enregistrement
    toast: Option<(String, Instant)>,
    rename_dlg: Option<(PathBuf, String)>,
    sidebar_open: bool,
    set_cat: SetCat,
    show_welcome: bool,
    shown_tab: Tab,      // onglet affiché à l'image précédente
    tab_at: Instant,     // moment du dernier changement d'onglet (pour le fondu)
    grid_sig: u64,       // filtres de la grille (dossier, recherche, tri…) à l'image précédente
    grid_at: Instant,    // départ de l'apparition en cascade des miniatures
    shown_view: Option<usize>, // capture affichée à l'image précédente
    view_at: Instant,    // ouverture / changement de capture dans la visionneuse
    view_dir: f32,       // -1 : précédente, 1 : suivante, 0 : ouverture depuis la grille
    shown_cat: SetCat,   // rubrique des paramètres affichée à l'image précédente
    cat_at: Instant,     // changement de rubrique des paramètres
    sel_shown: usize,    // nombre affiché dans la barre de sélection (gardé pendant qu'elle se replie)
}

pub fn run(settings: bool) {
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1100.0, 700.0])
        .with_min_inner_size([620.0, 420.0]);
    if let Ok(i) = image::load_from_memory(include_bytes!("../../assets/logo.png")) {
        let i = i.resize_exact(64, 64, image::imageops::FilterType::Lanczos3).to_rgba8();
        viewport = viewport.with_icon(egui::IconData { rgba: i.into_raw(), width: 64, height: 64 });
    }
    let opts = eframe::NativeOptions { viewport, ..Default::default() };
    let _ = eframe::run_native(
        "Shirokap",
        opts,
        Box::new(move |cc| {
            use raw_window_handle::{HasWindowHandle, RawWindowHandle};
            let hwnd = cc
                .window_handle()
                .ok()
                .and_then(|h| match h.as_raw() {
                    RawWindowHandle::Win32(w) => Some(w.hwnd.get()),
                    _ => None,
                })
                .unwrap_or(0);
            Ok(Box::new(App::new(&cc.egui_ctx, settings, hwnd)))
        }),
    );
}

// ---------- application ----------
impl App {
    fn new(ctx: &egui::Context, settings: bool, hwnd: isize) -> Self {
        let cfg = Config::load();
        let ui_state = load_ui();
        install_fonts(ctx);
        apply_style(ctx, &cfg);
        Self {
            items: list_items(&cfg.save_dir),
            saved: toml::to_string(&cfg).unwrap_or_default(),
            applied: (cfg.theme.clone(), cfg.accent_rgb()),
            shown_pal: palette(&cfg.theme),
            shown_acc: accent(&cfg),
            theme_anim: None,
            reveal_req: None,
            cmd: None,
            vis_cache: None,
            thumb_rects: HashMap::new(),
            fly_from: None,
            fly: None,
            view_rect: None,
            hwnd,
            look_key: None,
            reveal: None,
            applied_lang: cfg.language.clone(),
            fonts_cjk: lang() == ZH,
            cfg,
            tab: if settings { Tab::Settings } else { Tab::Gallery },
            favs: load_favs(),
            sel: HashSet::new(),
            folder: String::new(),
            search: String::new(),
            sort: match ui_state.sort {
                1 => Sort::Oldest,
                2 => Sort::Name,
                3 => Sort::Size,
                _ => Sort::Newest,
            },
            favs_only: ui_state.favs_only,
            thumb_w: ui_state.thumb_w.clamp(120.0, 420.0),
            thumbs: HashMap::new(),
            thumb_born: HashMap::new(),
            view: None,
            full: None,
            zoom: Default::default(),
            autostart: autostart_enabled(),
            focused: true,
            status: String::new(),
            stat: (0, 0),
            rename: None,
            slideshow: false,
            slide_at: Instant::now(),
            slide_new_first: ui_state.slide_new_first,
            fullscreen: false,
            editor: None,
            compare: None,
            recording: None,
            toast: None,
            rename_dlg: None,
            sidebar_open: ui_state.sidebar,
            set_cat: SetCat::Shortcuts,
            show_welcome: !ui_state.welcomed,
            shown_tab: Tab::Gallery,
            tab_at: Instant::now() - Duration::from_secs(1),
            grid_sig: 0,
            grid_at: Instant::now(),
            shown_view: None,
            view_at: Instant::now(),
            view_dir: 0.0,
            shown_cat: SetCat::Shortcuts,
            cat_at: Instant::now() - Duration::from_secs(1),
            sel_shown: 0,
        }
    }

    /// Captures affichées et leurs groupes par jour, gardées en cache tant que rien ne change
    /// (captures, favoris, recherche, dossier, tri, jour, langue) : inutile de tout refaire à chaque image.
    fn vis_groups(&mut self) -> std::rc::Rc<VisGroups> {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        self.items.len().hash(&mut h);
        for it in &self.items {
            it.size.hash(&mut h);
            it.modified.hash(&mut h);
        }
        self.items.first().map(|i| &i.path).hash(&mut h);
        self.favs.len().hash(&mut h);
        if self.favs_only {
            let mut f: Vec<&PathBuf> = self.favs.iter().collect();
            f.sort();
            f.hash(&mut h);
        }
        (&self.search, &self.folder, self.sort as u8, self.favs_only, crate::i18n::lang()).hash(&mut h);
        chrono::Local::now().date_naive().hash(&mut h);
        let key = h.finish();
        if let Some((k, c)) = &self.vis_cache {
            if *k == key {
                return c.clone();
            }
        }
        let vis = self.visible();
        let by_date = matches!(self.sort, Sort::Newest | Sort::Oldest);
        let mut groups: Vec<(String, Vec<usize>)> = Vec::new();
        for &i in &vis {
            let label = if by_date { day_label(self.items[i].modified) } else { String::new() };
            if groups.last().map(|g| g.0 == label).unwrap_or(false) {
                groups.last_mut().unwrap().1.push(i);
            } else {
                groups.push((label, vec![i]));
            }
        }
        let c = std::rc::Rc::new(VisGroups { vis, groups });
        self.vis_cache = Some((key, c.clone()));
        c
    }

    fn visible(&self) -> Vec<usize> {
        let q = self.search.to_lowercase();
        let mut v: Vec<usize> = (0..self.items.len())
            .filter(|&i| {
                let it = &self.items[i];
                (self.folder.is_empty() || it.folder == self.folder)
                    && (!self.favs_only || self.favs.contains(&it.path))
                    && (q.is_empty() || file_name(&it.path).to_lowercase().contains(&q))
            })
            .collect();
        match self.sort {
            Sort::Newest => {}
            Sort::Oldest => v.reverse(),
            Sort::Name => v.sort_by_key(|&i| file_name(&self.items[i].path).to_lowercase()),
            Sort::Size => v.sort_by_key(|&i| std::cmp::Reverse(self.items[i].size)),
        }
        v
    }

    fn toggle_fav(&mut self, p: PathBuf) {
        if !self.favs.remove(&p) {
            self.favs.insert(p);
        }
        save_favs(&self.favs);
    }

    fn trash(&mut self, paths: Vec<PathBuf>) {
        for p in &paths {
            if trash::delete(p).is_ok() {
                let _ = std::fs::remove_file(thumb_path(p));
                self.thumbs.remove(p);
                self.favs.remove(p);
                self.sel.remove(p);
            }
        }
        let before = self.items.len();
        self.items.retain(|it| it.path.exists());
        let gone = before - self.items.len();
        if gone > 0 {
            self.status = ic(icons::TRASH, &trf("{0} capture(s) mise(s) à la corbeille", &[&gone]));
        }
        self.full = None;
        save_favs(&self.favs);
    }

    fn hotkey_field(&mut self, idx: usize) -> &mut String {
        match idx {
            0 => &mut self.cfg.hotkey,
            1 => &mut self.cfg.hotkey_region,
            2 => &mut self.cfg.hotkey_window,
            3 => &mut self.cfg.hotkey_last_region,
            4 => &mut self.cfg.hotkey_gallery,
            5 => &mut self.cfg.hotkey_region_clip,
            _ => &mut self.cfg.hotkey_picker,
        }
    }

    fn toggle_fullscreen(&mut self, ctx: &egui::Context) {
        self.fullscreen = !self.fullscreen;
        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.fullscreen));
    }

    fn edit(&self, p: &Path) {
        let ed = if self.cfg.editor.trim().is_empty() { "mspaint" } else { self.cfg.editor.trim() };
        let _ = Command::new(ed).arg(p).spawn();
    }

    fn rename_file(&mut self, old: &Path, new_stem: &str) {
        let stem = clean(new_stem, 120);
        if stem.is_empty() {
            return;
        }
        let ext = old.extension().and_then(|e| e.to_str()).unwrap_or("png");
        let newp = old.with_file_name(format!("{stem}.{ext}"));
        if newp.exists() {
            self.status = ic(icons::ALERT, tr("Ce nom existe déjà"));
            return;
        }
        if std::fs::rename(old, &newp).is_ok() {
            let _ = std::fs::remove_file(thumb_path(old));
            self.thumbs.remove(old);
            if self.favs.remove(old) {
                self.favs.insert(newp.clone());
                save_favs(&self.favs);
            }
            self.sel.remove(old);
            self.items = list_items(&self.cfg.save_dir);
            if self.view.is_some() {
                self.view = self.items.iter().position(|it| it.path == newp);
            }
            self.full = None;
            self.status = ic(icons::RENAME, tr("Capture renommée"));
        }
    }

    /// Mémorise l'état de l'interface (taille des miniatures, tri, menu, accueil vu).
    fn save_state(&self) {
        save_ui(&UiState {
            thumb_w: self.thumb_w,
            sort: match self.sort {
                Sort::Newest => 0,
                Sort::Oldest => 1,
                Sort::Name => 2,
                Sort::Size => 3,
            },
            favs_only: self.favs_only,
            sidebar: self.sidebar_open,
            welcomed: !self.show_welcome,
            slide_new_first: self.slide_new_first,
        });
    }
}

impl eframe::App for App {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.save_state();
    }


    fn update(&mut self, ctx: &egui::Context, _f: &mut eframe::Frame) {
        frame_cap(ctx); // fluide quand ça bouge, économe sinon

        // F11 : plein écran depuis n'importe quel écran ; Échap en sort (hors visionneuse, qui gère son propre Échap)
        if ctx.input(|i| i.key_pressed(Key::F11)) {
            self.toggle_fullscreen(ctx);
        }
        if self.fullscreen
            && self.view.is_none()
            && self.editor.is_none()
            && self.compare.is_none()
            && ctx.input(|i| i.key_pressed(Key::Escape))
        {
            self.toggle_fullscreen(ctx);
        }
        // Ctrl+B : ouvrir / réduire le menu latéral
        if ctx.input(|i| i.modifiers.command && i.key_pressed(Key::B)) && !ctx.wants_keyboard_input() {
            self.sidebar_open = !self.sidebar_open;
        }
        // les messages d'état deviennent de petites notifications éphémères
        if !self.status.is_empty() {
            self.toast = Some((std::mem::take(&mut self.status), Instant::now()));
        }
        let toast = self.toast.as_ref().map(|(m, t)| (m.clone(), t.elapsed()));
        match toast {
            Some((_, age)) if age > Duration::from_millis(3200) => self.toast = None,
            Some((msg, age)) => {
                // la notification glisse vers le haut en apparaissant, puis s'efface en fondu
                let age = age.as_secs_f32();
                let enter = ease_out((age / anim::ENTER).min(1.0));
                let leave = ((3.2 - age) / 0.45).clamp(0.0, 1.0);
                if age < anim::ENTER || age > 2.75 {
                    ctx.anim_repaint();
                } else {
                    ctx.request_repaint_after(Duration::from_secs_f32(2.75 - age));
                }
                egui::Area::new(egui::Id::new("toast"))
                    .anchor(Align2::RIGHT_BOTTOM, vec2(-22.0 + 24.0 * (1.0 - leave), -56.0 + 40.0 * (1.0 - enter)))
                    .order(egui::Order::Foreground)
                    .interactable(false)
                    .show(ctx, |ui| {
                        ui.multiply_opacity(enter.min(leave));
                        egui::Frame::popup(ui.style()).inner_margin(egui::Margin::symmetric(16.0, 10.0)).show(ui, |ui| {
                            // un message qui commence par une icône : l'icône est mise en couleur (réussite, erreur…)
                            let mut chars = msg.chars();
                            match chars.next() {
                                Some(c) if ('\u{f700}'..='\u{f7ff}').contains(&c) => {
                                    let icon = c.to_string();
                                    let color = if icon == icons::ALERT {
                                        col(239, 68, 68)
                                    } else if icon == icons::TRASH {
                                        col(248, 113, 113)
                                    } else if icon == icons::SUCCESS {
                                        col(34, 197, 94)
                                    } else {
                                        self.shown_acc
                                    };
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new(icon).size(17.0).color(color));
                                        ui.label(chars.as_str().trim_start());
                                    });
                                }
                                _ => {
                                    ui.label(msg);
                                }
                            }
                        });
                    });
            }
            None => {}
        }
        // fenêtre « Renommer » (depuis le menu clic droit de la galerie)
        let (mut do_rename, mut close) = (None::<(PathBuf, String)>, false);
        if let Some((path, buf)) = &mut self.rename_dlg {
            egui::Window::new(tr("Renommer la capture"))
                .collapsible(false)
                .resizable(false)
                .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.weak(file_name(path.as_path()));
                    let r = ui.add(egui::TextEdit::singleline(buf).desired_width(360.0));
                    r.request_focus();
                    let enter = r.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
                    ui.horizontal(|ui| {
                        if ui.button(tr("Renommer")).clicked() || enter {
                            do_rename = Some((path.clone(), buf.clone()));
                        }
                        if ui.button(tr("Annuler")).clicked() {
                            close = true;
                        }
                    });
                });
            if ctx.input(|i| i.key_pressed(Key::Escape)) {
                close = true;
            }
        }
        if close {
            self.rename_dlg = None;
        }
        if let Some((p, n)) = do_rename {
            self.rename_dlg = None;
            self.rename_file(&p, &n);
        }
        // écran d'accueil (au premier lancement, ou via Paramètres > Système)
        if self.show_welcome && self.editor.is_none() {
            let (mut start, mut custom, mut change) = (false, false, false);
            let mut keys: Vec<(String, &'static str)> = Vec::new();
            for (k, d) in [
                (self.cfg.hotkey.clone(), tr("Capturer l'écran")),
                (self.cfg.hotkey_region.clone(), tr("Capturer une zone")),
                (self.cfg.hotkey_window.clone(), tr("Capturer la fenêtre active")),
                (self.cfg.hotkey_gallery.clone(), tr("Ouvrir la galerie")),
            ] {
                if !k.is_empty() {
                    keys.push((k, d));
                }
            }
            let dir = self.cfg.save_dir.clone();
            let acc = self.shown_acc;
            egui::Window::new(tr("Bienvenue"))
                .title_bar(false)
                .collapsible(false)
                .resizable(false)
                .default_width(560.0)
                .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.add_space(6.0);
                    ui.label(bold_text(tr("Bienvenue dans Shirokap"), 26.0));
                    ui.weak(tr("Une capture d'écran légère, qui tourne discrètement dans la zone de notification."));
                    ui.add_space(12.0);
                    ui.label(bold_text(tr("Tes raccourcis"), 16.0));
                    for (k, d) in &keys {
                        ui.horizontal(|ui| {
                            let pill = ui.visuals().widgets.inactive.bg_fill;
                            egui::Frame::none().fill(pill).rounding(6.0_f32).inner_margin(egui::Margin::symmetric(9.0, 3.0)).show(ui, |ui| {
                                ui.label(bold_text(k, 14.0));
                            });
                            ui.label(*d);
                        });
                    }
                    ui.add_space(8.0);
                    ui.label(bold_text(tr("Dans la zone de notification"), 16.0));
                    ui.label(tr("Clic gauche sur l'icône : ouvrir la galerie  ·  clic molette : capturer  ·  clic droit : menu."));
                    ui.add_space(8.0);
                    ui.label(bold_text(tr("Tes captures"), 16.0));
                    ui.horizontal_wrapped(|ui| {
                        ui.weak(&dir);
                        if ui.button(tr("Changer…")).clicked() {
                            change = true;
                        }
                    });
                    ui.add_space(14.0);
                    ui.horizontal(|ui| {
                        let go = egui::Button::new(egui::RichText::new(tr("C'est parti")).color(on_accent(acc))).fill(acc);
                        if ui.add(go).clicked() {
                            start = true;
                        }
                        if ui.button(tr("Personnaliser les raccourcis")).clicked() {
                            custom = true;
                        }
                    });
                    ui.add_space(4.0);
                });
            if change {
                if let Some(p) = rfd::FileDialog::new().pick_folder() {
                    self.cfg.save_dir = p.display().to_string();
                    self.cfg.save();
                    self.saved = toml::to_string(&self.cfg).unwrap_or_default();
                    self.items = list_items(&self.cfg.save_dir);
                }
            }
            if custom {
                self.tab = Tab::Settings;
                self.set_cat = SetCat::Shortcuts;
            }
            if start || custom {
                self.show_welcome = false;
                self.save_state();
            }
        }
        if let Some(idx) = self.recording {
            ctx.anim_repaint(); // on lit le clavier en continu pendant l'enregistrement
            match poll_hotkey() {
                Poll::Wait => {}
                Poll::Cancel => self.recording = None,
                Poll::Clear => {
                    self.hotkey_field(idx).clear();
                    self.recording = None;
                }
                Poll::Combo(c) => {
                    *self.hotkey_field(idx) = c;
                    self.recording = None;
                }
            }
        }
        if self.cfg.language != self.applied_lang {
            set_lang(&self.cfg.language); // aperçu en direct de la langue
            self.applied_lang = self.cfg.language.clone();
        }
        // police chinoise : pour l'interface en chinois, la page où l'on choisit la langue
        // (le nom « 中文 » doit s'afficher dans toutes les langues) et l'éditeur (texte saisi)
        let need_cjk = lang() == ZH
            || (self.tab == Tab::Settings && self.set_cat == SetCat::System)
            || self.editor.is_some();
        if need_cjk != self.fonts_cjk {
            install_fonts_with(ctx, need_cjk);
            self.fonts_cjk = need_cjk;
        }
        // Changement de thème : un cercle part du clic et grandit en révélant le nouveau thème.
        // On photographie d'abord l'écran avec l'ancien thème (la capture arrive à l'image suivante),
        // puis on applique le nouveau dessous et on dessine la photo par-dessus, percée d'un cercle qui grandit.
        // Changement de la seule couleur d'accent : fondu des couleurs.
        let cur = (self.cfg.theme.clone(), self.cfg.accent_rgb());
        if cur != self.applied && self.reveal_req.is_none() {
            if cur.0 != self.applied.0 {
                let origin = ctx.input(|i| i.pointer.interact_pos().or(i.pointer.hover_pos())).unwrap_or(ctx.screen_rect().center());
                // un cercle était encore en cours (clics rapides) : on le termine d'un coup pour que la photo
                // montre le thème précédent en entier, sans frontière ni liseré figés
                self.reveal = None;
                self.reveal_req = Some((origin, Instant::now()));
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot);
                ctx.anim_repaint();
            } else {
                self.theme_anim = Some((self.shown_pal, self.shown_acc, Instant::now()));
                self.applied = cur;
            }
        }
        if let Some((origin, asked)) = self.reveal_req {
            let shot = ctx.input(|i| {
                i.raw.events.iter().find_map(|e| match e {
                    egui::Event::Screenshot { image, .. } => Some(image.clone()),
                    _ => None,
                })
            });
            if shot.is_some() || asked.elapsed() > Duration::from_millis(400) {
                if let Some(img) = shot {
                    let tex = ctx.load_texture("theme_reveal", (*img).clone(), egui::TextureOptions::LINEAR);
                    self.reveal = Some((tex, origin, Instant::now()));
                    self.theme_anim = None;
                    self.shown_pal = palette(&self.cfg.theme);
                    self.shown_acc = accent(&self.cfg);
                    apply_style_with(ctx, &self.shown_pal, self.shown_acc);
                } else {
                    // pas de capture possible (pilote graphique) : simple fondu
                    self.theme_anim = Some((self.shown_pal, self.shown_acc, Instant::now()));
                }
                self.applied = (self.cfg.theme.clone(), self.cfg.accent_rgb());
                self.reveal_req = None;
            }
            ctx.anim_repaint();
        }
        if let Some((from_pal, from_acc, t0)) = self.theme_anim {
            let t = egui::emath::easing::cubic_in_out((t0.elapsed().as_secs_f32() / anim::THEME).min(1.0));
            self.shown_pal = lerp_pal(&from_pal, &palette(&self.cfg.theme), t);
            self.shown_acc = from_acc.lerp_to_gamma(accent(&self.cfg), t);
            apply_style_with(ctx, &self.shown_pal, self.shown_acc);
            if t >= 1.0 {
                self.theme_anim = None;
            } else {
                ctx.anim_repaint();
            }
        }
        // barre de titre de Windows : claire ou sombre, et de la couleur du fond du thème (Windows 11)
        let target_pal = palette(&self.cfg.theme);
        let key = (target_pal.dark, [target_pal.bg.r(), target_pal.bg.g(), target_pal.bg.b()]);
        if self.look_key != Some(key) && self.reveal_req.is_none() {
            self.look_key = Some(key);
            title_bar_look(self.hwnd, target_pal.dark, target_pal.bg);
        }
        let foc = ctx.input(|i| i.focused);
        if foc && !self.focused {
            self.items = list_items(&self.cfg.save_dir); // nouvelles captures au retour sur la fenêtre
        }
        self.focused = foc;

        let immersive = self.fullscreen && self.view.is_some() && self.editor.is_none() && self.compare.is_none();
        if !immersive {
        let side_fill = self.shown_pal.side;
        // le menu latéral s'ouvre et se replie en glissant (56 px replié, 190 px ouvert)
        let open_k = ctx.anim_be(egui::Id::new("nav_open"), self.sidebar_open, anim::SIDEBAR, egui::emath::easing::cubic_in_out);
        let nav_w = 56.0 + (190.0 - 56.0) * open_k;
        let wide = nav_w > 150.0;
        egui::SidePanel::left("nav")
            .exact_width(nav_w)
            .resizable(false)
            .frame(egui::Frame::none().fill(side_fill).inner_margin(egui::Margin::same(if wide { 14.0 } else { 11.0 })))
            .show(ctx, |ui| {
                if wide {
                    ui.multiply_opacity(((nav_w - 150.0) / 40.0).clamp(0.0, 1.0));
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.label(bold_text("Shirokap", 22.0));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if sidebar_icon_button(ui).on_hover_text(tr("Réduire le menu (Ctrl+B)")).clicked() {
                                self.sidebar_open = false;
                            }
                        });
                    });
                    ui.add_space(14.0);
                    nav_item(ui, &mut self.tab, Tab::Gallery, icons::GALLERY, tr("Galerie"));
                    nav_item(ui, &mut self.tab, Tab::Stats, icons::STATS, tr("Statistiques"));
                    nav_item(ui, &mut self.tab, Tab::Settings, icons::SETTINGS, tr("Paramètres"));
                    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                        ui.add_space(4.0);
                        ui.weak(trf("Version {0}", &[&env!("CARGO_PKG_VERSION")]));
                        ui.add_space(6.0);
                        let label = if self.fullscreen {
                            ic(icons::EXIT_FULLSCREEN, tr("Quitter le plein écran"))
                        } else {
                            ic(icons::FULLSCREEN, tr("Plein écran"))
                        };
                        if ui.add(egui::Button::new(label).shortcut_text("F11")).clicked() {
                            self.toggle_fullscreen(ctx);
                        }
                    });
                } else {
                    // menu réduit : une fine colonne d'icônes (onglets + plein écran)
                    ui.add_space(6.0);
                    if sidebar_icon_button(ui).on_hover_text(tr("Ouvrir le menu (Ctrl+B)")).clicked() {
                        self.sidebar_open = true;
                    }
                    ui.add_space(14.0);
                    for (t, icon, name) in [
                        (Tab::Gallery, icons::GALLERY, tr("Galerie")),
                        (Tab::Stats, icons::STATS, tr("Statistiques")),
                        (Tab::Settings, icons::SETTINGS, tr("Paramètres")),
                    ] {
                        if icon_button(ui, icon, self.tab == t).on_hover_text(name).clicked() {
                            self.tab = t;
                        }
                    }
                    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                        ui.add_space(4.0);
                        let (icon, tip) = if self.fullscreen {
                            (icons::EXIT_FULLSCREEN, tr("Quitter le plein écran"))
                        } else {
                            (icons::FULLSCREEN, tr("Plein écran"))
                        };
                        if icon_button(ui, icon, false).on_hover_text(format!("{tip} (F11)")).clicked() {
                            self.toggle_fullscreen(ctx);
                        }
                    });
                }
            });

        match self.tab {
            Tab::Gallery => {
                egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.weak(format!("{} · {}", n_captures(self.stat.0), human(self.stat.1)));
                        ui.weak(&self.status);
                    });
                });
            }
            Tab::Stats => {}
            Tab::Settings => {
                let ok = hotkeys_ok(&self.cfg);
                let dirty = toml::to_string(&self.cfg).unwrap_or_default() != self.saved;
                egui::TopBottomPanel::bottom("save").show(ctx, |ui| {
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        // « Enregistrer » prend la couleur d'accent quand il y a quelque chose à enregistrer
                        let acc = self.shown_acc;
                        let save_txt = ic(icons::SAVE, tr("Enregistrer"));
                        let save_btn = if dirty && ok {
                            egui::Button::new(egui::RichText::new(save_txt).color(on_accent(acc))).fill(acc)
                        } else {
                            egui::Button::new(save_txt)
                        };
                        if ui.add_enabled(dirty && ok, save_btn).clicked() {
                            self.cfg.save(); // le démon relit la config tout seul
                            self.saved = toml::to_string(&self.cfg).unwrap_or_default();
                            self.items = list_items(&self.cfg.save_dir);
                            self.status = ic(icons::SUCCESS, tr("Enregistré ✔").trim_end_matches('✔').trim_end());
                        }
                        if ui.add_enabled(dirty, egui::Button::new(ic(icons::UNDO, tr("Annuler")))).clicked() {
                            self.cfg = Config::load();
                        }
                        if ui
                            .button(ic(icons::RESET, tr("Tout réinitialiser")))
                            .on_hover_text(tr("Remet TOUS les réglages de TOUTES les rubriques à leur valeur d'origine. Rien n'est appliqué tant que tu n'as pas cliqué sur « Enregistrer » (« Annuler » permet de revenir en arrière)."))
                            .clicked()
                        {
                            self.cfg = Config::default();
                        }
                        if !ok {
                            ui.colored_label(Color32::LIGHT_RED, tr("Un raccourci est invalide"));
                        } else if dirty {
                            ui.colored_label(Color32::GOLD, tr("Modifications non enregistrées"));
                        } else {
                            ui.weak(&self.status);
                        }
                    });
                    ui.add_space(4.0);
                });
            }
        }

        }
        let content = if immersive {
            egui::Frame::none().fill(Color32::BLACK)
        } else {
            egui::Frame::none()
                .fill(ctx.style().visuals.panel_fill)
                .inner_margin(egui::Margin::symmetric(22.0, 16.0))
        };
        // palette de commandes : son clavier passe avant les autres raccourcis
        let pending = self.palette_keys(ctx);
        self.run_pending_cmd(ctx, pending);

        // changement d'onglet : le contenu apparaît en fondu en remontant
        if self.tab != self.shown_tab {
            self.shown_tab = self.tab;
            self.tab_at = Instant::now();
            if self.tab == Tab::Gallery {
                self.grid_at = Instant::now(); // les miniatures réapparaissent en cascade
            }
        }
        // visionneuse : on note quand on ouvre une capture ou qu'on en change ; au retour à la grille, fondu
        if self.view != self.shown_view {
            if self.view.is_some() {
                self.view_at = Instant::now();
                self.zoom = Default::default(); // chaque capture s'ouvre en entier
            } else {
                self.tab_at = Instant::now();
                // fermeture : l'image rejoint sa miniature (ou rétrécit en s'effaçant si elle n'est pas à l'écran)
                if let (Some(i), Some(from), Some((p, tex))) = (self.shown_view, self.view_rect, self.full.as_ref()) {
                    if self.items.get(i).map(|it| &it.path == p).unwrap_or(false) {
                        let to = self.thumb_rects.get(&i).copied();
                        self.fly = Some(Fly {
                            tex: tex.clone(),
                            from,
                            to: to.unwrap_or_else(|| Rect::from_center_size(from.center(), from.size() * 0.6)),
                            t0: Instant::now(),
                            closing: true,
                            fade: to.is_none(),
                            idx: i,
                        });
                    }
                }
                self.view_rect = None;
                self.fly_from = None;
            }
            self.shown_view = self.view;
        }
        let tab_k = ease_out((self.tab_at.elapsed().as_secs_f32() / anim::PAGE).min(1.0));
        if tab_k < 1.0 {
            ctx.anim_repaint();
        }
        egui::CentralPanel::default().frame(content).show(ctx, |ui| {
            ui.multiply_opacity(tab_k);
            ui.add_space(24.0 * (1.0 - tab_k));
            match self.tab {
            Tab::Gallery => self.gallery_ui(ui),
            Tab::Settings => self.settings_ui(ui),
            Tab::Stats => self.stats_ui(ui),
            }
        });

        // l'image qui vole entre la miniature et la visionneuse
        if let Some(f) = &self.fly {
            let t = (f.t0.elapsed().as_secs_f32() / anim::FLY).min(1.0);
            let e = egui::emath::easing::cubic_in_out(t);
            let rect = f.from.lerp_towards(&f.to, e);
            let round = if f.closing { 10.0 * e } else { 10.0 * (1.0 - e) };
            let alpha = if f.fade { 1.0 - e } else { 1.0 };
            let mut shape = egui::epaint::RectShape::filled(rect, round, Color32::from_white_alpha((alpha * 255.0) as u8));
            shape.fill_texture_id = f.tex.id();
            shape.uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
            let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("fly")));
            if !f.fade {
                let sh = egui::epaint::Shadow { offset: vec2(0.0, 10.0), blur: 30.0, spread: 0.0, color: Color32::from_black_alpha((120.0 * (1.0 - (2.0 * e - 1.0).abs())) as u8) };
                painter.add(sh.as_shape(rect, round));
            }
            painter.add(shape);
            if t >= 1.0 {
                self.fly = None;
            } else {
                ctx.anim_repaint();
            }
        }
        self.palette_ui(ctx);

        // par-dessus tout : l'ancien thème, percé d'un cercle qui grandit depuis le clic
        if let Some((tex, origin, t0)) = &self.reveal {
            let t = (t0.elapsed().as_secs_f32() / anim::REVEAL).min(1.0);
            let e = egui::emath::easing::cubic_in_out(t);
            let screen = ctx.screen_rect();
            let far = [screen.left_top(), screen.right_top(), screen.left_bottom(), screen.right_bottom()]
                .iter()
                .map(|c| c.distance(*origin))
                .fold(0.0_f32, f32::max);
            let r = e * (far + 4.0);
            let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Tooltip, egui::Id::new("theme_reveal")));
            painter.add(egui::Shape::mesh(reveal_mesh(tex.id(), screen, *origin, r, far * 1.6 + 200.0)));
            // fin liseré lumineux à la frontière, aux couleurs du nouveau thème
            if r > 1.0 {
                painter.circle_stroke(*origin, r, Stroke::new(2.0_f32, self.shown_acc.gamma_multiply(0.8 * (1.0 - t))));
            }
            if t >= 1.0 {
                self.reveal = None; // la capture est libérée
            } else {
                ctx.anim_repaint();
            }
        }
    }
}

/// Toute la fenêtre sauf un disque (centre `c`, rayon `r`), texturée avec la capture de l'écran.
fn reveal_mesh(tex: egui::TextureId, screen: Rect, c: Pos2, r: f32, outer: f32) -> egui::Mesh {
    let mut m = egui::Mesh::with_texture(tex);
    let uv = |p: Pos2| pos2((p.x - screen.min.x) / screen.width(), (p.y - screen.min.y) / screen.height());
    let n = 120u32;
    for i in 0..=n {
        let a = i as f32 / n as f32 * std::f32::consts::TAU;
        let d = vec2(a.cos(), a.sin());
        for p in [c + d * r, c + d * outer] {
            m.vertices.push(egui::epaint::Vertex { pos: p, uv: uv(p), color: Color32::WHITE });
        }
    }
    for i in 0..n {
        let k = 2 * i;
        m.add_triangle(k, k + 1, k + 2);
        m.add_triangle(k + 1, k + 3, k + 2);
    }
    m
}

/// Durées des animations, en secondes.
pub(crate) mod anim {
    /// survol d'un élément (miniature, bouton, menu)
    pub const HOVER: f32 = 0.25;
    /// apparition d'un contenu (miniature, notification, image)
    pub const ENTER: f32 = 0.45;
    /// changement d'onglet ou de rubrique
    pub const PAGE: f32 = 0.38;
    /// ouverture / repli du menu latéral
    pub const SIDEBAR: f32 = 0.35;
    /// décalage entre deux miniatures dans l'apparition en cascade
    pub const STAGGER: f32 = 0.04;
    /// vol d'une capture entre sa miniature et la visionneuse
    pub const FLY: f32 = 0.42;
    /// changement de couleur d'accent (fondu)
    pub const THEME: f32 = 0.4;
    /// changement de thème (cercle qui révèle le nouveau thème)
    pub const REVEAL: f32 = 0.8;
}

/// Captures affichées (indices dans `items`) et leurs groupes par jour.
struct VisGroups {
    vis: Vec<usize>,
    groups: Vec<(String, Vec<usize>)>,
}

/// Image qui « vole » entre sa miniature et la visionneuse (ouverture et fermeture).
struct Fly {
    tex: TextureHandle,
    from: Rect,
    to: Rect,
    t0: Instant,
    closing: bool,
    fade: bool, // fermeture sans miniature visible : l'image rétrécit en s'effaçant
    idx: usize,
}

/// Courbe d'animation : rapide au début, douce à la fin.
pub(crate) fn ease_out(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

/// Barre de titre assortie au thème : mode sombre ou clair, et couleur du fond du thème
/// (la couleur n'est prise en compte que par Windows 11 ; Windows 10 garde sa barre habituelle).
fn title_bar_look(hwnd: isize, dark: bool, bg: Color32) {
    use windows_sys::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_CAPTION_COLOR, DWMWA_USE_IMMERSIVE_DARK_MODE};
    if hwnd == 0 {
        return;
    }
    let h = hwnd as windows_sys::Win32::Foundation::HWND;
    let dark_flag: i32 = dark as i32;
    let c: u32 = bg.r() as u32 | (bg.g() as u32) << 8 | (bg.b() as u32) << 16; // COLORREF : 0x00BBGGRR
    unsafe {
        DwmSetWindowAttribute(h, DWMWA_USE_IMMERSIVE_DARK_MODE as u32, &dark_flag as *const i32 as _, 4);
        DwmSetWindowAttribute(h, DWMWA_CAPTION_COLOR as u32, &c as *const u32 as _, 4);
    }
}
