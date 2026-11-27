use crate::common::*;
use crate::i18n::*;
use global_hotkey::{hotkey::HotKey, GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use image::RgbaImage;
use std::{
    collections::HashMap,
    ffi::c_void,
    mem::size_of,
    path::PathBuf,
    process::{Child, Command},
    sync::mpsc,
    time::{Duration, Instant, SystemTime},
};
use tao::event_loop::{ControlFlow, EventLoop};
use tray_icon::{
    menu::{CheckMenuItem, ContextMenu, Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem, Submenu},
    Icon, MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent,
};
use windows_sys::Win32::{
    Foundation::{CloseHandle, GetLastError, POINT, RECT},
    Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS},
    Graphics::Gdi::InvalidateRect,
    System::Threading::{CreateMutexW, OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION},
    UI::WindowsAndMessaging::{
        AllowSetForegroundWindow, EnumWindows, FindWindowW, GetCursorPos, GetForegroundWindow, GetWindowRect, GetWindowTextW,
        GetWindowThreadProcessId, IsWindowVisible, PostMessageW, SetForegroundWindow, ShowWindow, SW_RESTORE, WM_CLOSE,
    },
};

// capture, zone, fenêtre, dernière zone, galerie, zone→presse-papiers, pipette
type Keys = [Option<HotKey>; 7];
const GRAY: [u8; 3] = [128, 128, 128];
const GREEN: [u8; 3] = [34, 197, 94];
const RES: [(u32, &str); 5] = [
    (0, "Résolution native"),
    (3840, "4K · 3840 px"),
    (2560, "1440p · 2560 px"),
    (1920, "1080p · 1920 px"),
    (1280, "720p · 1280 px"),
];

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Screen,
    Window,
    LastZone,
}

#[derive(Clone, Copy, PartialEq)]
enum Cmd {
    Gallery,
    Settings,
    Snap(Mode),
    Delayed(Mode),
    Zone,
    ZoneClip,
    Pick,
    PinLast,
    TrashLast,
    Auto,
    Last,
    CopyLast,
    Folder,
    ConfigDir,
    Pause,
    Quit,
    TogClip,
    TogSound,
    TogFlash,
    TogFolder,
    Res(u32),
}

fn already_running() -> bool {
    let name: Vec<u16> = "Local\\ShirokapDaemon".encode_utf16().chain([0]).collect();
    unsafe {
        CreateMutexW(std::ptr::null(), 0, name.as_ptr());
        GetLastError() == 183 // ERROR_ALREADY_EXISTS
    }
}

/// Icône du tray = assets/logo.png (état normal), en gris (pause) ou teintée de vert (capture faite).
fn icon(rgb: [u8; 3]) -> Icon {
    let mut img = image::load_from_memory(include_bytes!("../assets/logo.png"))
        .map(|i| i.resize_exact(32, 32, image::imageops::FilterType::Lanczos3).to_rgba8())
        .unwrap_or_else(|_| RgbaImage::new(32, 32));
    let mix = |c: u8, t: u8| ((c as u16 + t as u16) / 2) as u8;
    for p in img.pixels_mut() {
        let [r, g, b, _] = p.0;
        if rgb == GRAY {
            let l = ((r as u16 + g as u16 + b as u16) / 3) as u8;
            p.0[..3].copy_from_slice(&[l, l, l]);
        } else if rgb == GREEN {
            p.0[..3].copy_from_slice(&[mix(r, GREEN[0]), mix(g, GREEN[1]), mix(b, GREEN[2])]);
        }
    }
    Icon::from_rgba(img.into_raw(), 32, 32).unwrap()
}

fn tooltip(cfg: &Config, keys: &Keys, paused: bool, auto: bool) -> String {
    if paused {
        return tr("Shirokap (en pause)").into();
    }
    let names = [
        &cfg.hotkey,
        &cfg.hotkey_region,
        &cfg.hotkey_window,
        &cfg.hotkey_last_region,
        &cfg.hotkey_gallery,
        &cfg.hotkey_region_clip,
        &cfg.hotkey_picker,
    ];
    let bad = names.iter().zip(keys.iter()).any(|(n, k)| !n.is_empty() && k.is_none());
    format!(
        "Shirokap : {}{}{}",
        trf("{0} capture, {1} zone", &[&cfg.hotkey, &cfg.hotkey_region]),
        if auto { tr(" · timelapse actif") } else { "" },
        if bad { tr(" (un raccourci est indisponible)") } else { "" }
    )
}

fn rebind(m: &GlobalHotKeyManager, keys: &mut Keys, cfg: &Config) {
    for k in keys.iter_mut() {
        if let Some(h) = k.take() {
            let _ = m.unregister(h);
        }
    }
    let names = [
        &cfg.hotkey,
        &cfg.hotkey_region,
        &cfg.hotkey_window,
        &cfg.hotkey_last_region,
        &cfg.hotkey_gallery,
        &cfg.hotkey_region_clip,
        &cfg.hotkey_picker,
    ];
    for (k, s) in keys.iter_mut().zip(names) {
        match s.parse::<HotKey>() {
            Ok(h) => {
                if m.register(h).is_ok() {
                    *k = Some(h);
                } else {
                    log(&format!("hotkey unavailable (already used by another program?): {s}"));
                }
            }
            Err(_) => {
                if !s.is_empty() {
                    log(&format!("invalid hotkey: {s}"));
                }
            }
        }
    }
}

// ---------- menu du tray ----------
struct Items {
    header: MenuItem,
    today: MenuItem,
    snap: MenuItem,
    zone: MenuItem,
    zoneclip: MenuItem,
    pick: MenuItem,
    lastzone: MenuItem,
    gallery: MenuItem,
    auto: CheckMenuItem,
    pause: CheckMenuItem,
    clip: CheckMenuItem,
    sound: CheckMenuItem,
    flash: CheckMenuItem,
    folder: CheckMenuItem,
    res: Vec<(u32, CheckMenuItem)>,
}

fn mi(cmds: &mut HashMap<MenuId, Cmd>, text: &str, cmd: Cmd) -> MenuItem {
    let m = MenuItem::new(text, true, None);
    cmds.insert(m.id().clone(), cmd);
    m
}
fn ck(cmds: &mut HashMap<MenuId, Cmd>, text: &str, checked: bool, cmd: Cmd) -> CheckMenuItem {
    let m = CheckMenuItem::new(text, true, checked, None);
    cmds.insert(m.id().clone(), cmd);
    m
}
/// Élément du menu natif dont le texte suit la langue.
enum Lbl {
    M(MenuItem),
    C(CheckMenuItem),
    S(Submenu),
}

fn relabel(list: &[(Lbl, &'static str)]) {
    for (item, key) in list {
        match item {
            Lbl::M(m) => m.set_text(tr(key)),
            Lbl::C(c) => c.set_text(tr(key)),
            Lbl::S(s) => s.set_text(tr(key)),
        }
    }
}

fn keyed(label: &str, key: &str) -> String {
    if key.is_empty() {
        label.to_string()
    } else {
        format!("{label}    ({key})")
    }
}

/// Met les textes et les cases du menu en accord avec la config.
fn sync(it: &Items, cfg: &Config, today: usize) {
    it.header.set_text(format!("Shirokap {}", env!("CARGO_PKG_VERSION")));
    it.today.set_text(today_label(today));
    it.snap.set_text(keyed(tr("Écran entier"), &cfg.hotkey));
    it.zone.set_text(keyed(tr("Zone…"), &cfg.hotkey_region));
    it.zoneclip.set_text(keyed(tr("Zone → presse-papiers"), &cfg.hotkey_region_clip));
    it.pick.set_text(keyed(tr("Pipette : couleur sous le curseur (3 s)"), &cfg.hotkey_picker));
    it.lastzone.set_text(keyed(tr("Refaire la dernière zone"), &cfg.hotkey_last_region));
    it.gallery.set_text(keyed(tr("Ouvrir la galerie"), &cfg.hotkey_gallery));
    it.clip.set_checked(cfg.clipboard);
    it.sound.set_checked(cfg.sound);
    it.flash.set_checked(cfg.flash);
    it.folder.set_checked(cfg.folder_per_window);
    for (w, c) in &it.res {
        c.set_checked(*w == cfg.target_width);
    }
}

fn persist(cfg: &Config, mtime: &mut Option<SystemTime>) {
    cfg.save();
    *mtime = config_mtime(); // évite de recharger notre propre modification
}

// ---------- capture ----------
fn active_title() -> String {
    let mut buf = [0u16; 256];
    let n = unsafe { GetWindowTextW(GetForegroundWindow(), buf.as_mut_ptr(), 256) };
    String::from_utf16_lossy(&buf[..n.max(0) as usize])
}

fn grab_at(x: i32, y: i32) -> Option<RgbaImage> {
    xcap::Monitor::from_point(x, y).ok()?.capture_image().ok()
}

fn grab_last_zone() -> Option<RgbaImage> {
    let (x, y, w, h, mx, my) = load_last_region()?;
    let img = grab_at(mx, my)?; // l'écran où la zone avait été tracée
    if x + w > img.width() || y + h > img.height() {
        return None; // la résolution a changé depuis
    }
    Some(image::imageops::crop_imm(&img, x, y, w, h).to_image())
}

fn grab_all() -> Option<RgbaImage> {
    let mut shots = Vec::new();
    for m in xcap::Monitor::all().ok()? {
        if let (Ok(x), Ok(y), Ok(img)) = (m.x(), m.y(), m.capture_image()) {
            shots.push((x, y, img));
        }
    }
    let minx = shots.iter().map(|s| s.0).min()?;
    let miny = shots.iter().map(|s| s.1).min()?;
    let w = shots.iter().map(|s| s.0 - minx + s.2.width() as i32).max()? as u32;
    let h = shots.iter().map(|s| s.1 - miny + s.2.height() as i32).max()? as u32;
    let mut canvas = RgbaImage::new(w, h);
    for (x, y, img) in &shots {
        image::imageops::replace(&mut canvas, img, (*x - minx) as i64, (*y - miny) as i64);
    }
    Some(canvas)
}

fn grab_window() -> Option<RgbaImage> {
    let mut r = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    unsafe {
        let h = GetForegroundWindow();
        let ok = DwmGetWindowAttribute(
            h,
            DWMWA_EXTENDED_FRAME_BOUNDS as u32,
            &mut r as *mut RECT as *mut c_void,
            size_of::<RECT>() as u32,
        ) == 0;
        if !ok && GetWindowRect(h, &mut r) == 0 {
            return None;
        }
    }
    if r.right <= r.left || r.bottom <= r.top {
        return None;
    }
    let mon = xcap::Monitor::from_point((r.left + r.right) / 2, (r.top + r.bottom) / 2).ok()?;
    let (mx, my) = (mon.x().ok()?, mon.y().ok()?);
    let img = mon.capture_image().ok()?;
    let x = (r.left - mx).max(0) as u32;
    let y = (r.top - my).max(0) as u32;
    let w = ((r.right - r.left) as u32).min(img.width().saturating_sub(x));
    let h = ((r.bottom - r.top) as u32).min(img.height().saturating_sub(y));
    if w < 8 || h < 8 {
        return None;
    }
    Some(image::imageops::crop_imm(&img, x, y, w, h).to_image())
}

fn grab(cfg: &Config, mode: Mode) -> Option<RgbaImage> {
    match mode {
        Mode::Window => grab_window(),
        Mode::LastZone => grab_last_zone(),
        Mode::Screen => match cfg.monitor_mode.as_str() {
            "all" => grab_all(),
            "cursor" => {
                let mut p = POINT { x: 0, y: 0 };
                unsafe { GetCursorPos(&mut p) };
                grab_at(p.x, p.y)
            }
            _ => grab_at(0, 0), // l'écran principal contient toujours (0,0)
        },
    }
}

/// La capture est instantanée ; l'encodage (lourd en 4K) part dans un thread pour ne jamais bloquer le tray.
/// `quiet` = pas de clignotement du tray (timelapse).
fn shoot(cfg: &Config, mode: Mode, tx: &mpsc::Sender<bool>, quiet: bool) {
    let title = active_title();
    if is_blocked(&cfg.blocklist, &title) {
        return; // fenêtre sensible : on ne capture pas
    }
    // pendant une capture, on quitte le mode efficacité : la capture et l'encodage vont à pleine vitesse
    set_efficiency(false);
    let Some(img) = grab(cfg, mode) else {
        log("capture failed: the screen could not be read");
        set_efficiency(true);
        return;
    };
    let (cfg, tx) = (cfg.clone(), tx.clone());
    std::thread::spawn(move || {
        if finish(&cfg, opaque(img), &title).is_some() {
            let _ = tx.send(!quiet);
        }
        set_efficiency(true);
    });
}

/// Copie la couleur (#RRGGBB) du pixel sous le curseur dans le presse-papiers.
fn do_pick(it: &Items) -> bool {
    let hex = (|| -> Option<String> {
        let mut p = POINT { x: 0, y: 0 };
        unsafe { GetCursorPos(&mut p) };
        let mon = xcap::Monitor::from_point(p.x, p.y).ok()?;
        let (mx, my) = (mon.x().ok()?, mon.y().ok()?);
        let img = mon.capture_image().ok()?;
        let (x, y) = ((p.x - mx).max(0) as u32, (p.y - my).max(0) as u32);
        if x >= img.width() || y >= img.height() {
            return None;
        }
        let c = img.get_pixel(x, y).0;
        Some(format!("#{:02X}{:02X}{:02X}", c[0], c[1], c[2]))
    })();
    match hex {
        Some(h) => {
            copy_text(&h);
            it.today.set_text(trf("Couleur copiée : {0}", &[&h]));
            true
        }
        None => false,
    }
}

fn auto_cfg(cfg: &Config) -> Config {
    let mut c = cfg.clone();
    c.clipboard = false;
    c.sound = false;
    c.folder_per_window = false;
    c.save_dir = PathBuf::from(&cfg.save_dir).join("Timelapse").display().to_string();
    c
}

fn trigger(cfg: &Config, mode: Mode, tx: &mpsc::Sender<bool>, pending: &mut Option<(Instant, Mode)>) {
    if cfg.delay_secs == 0 {
        shoot(cfg, mode, tx, false);
    } else {
        *pending = Some((Instant::now() + Duration::from_secs(cfg.delay_secs as u64), mode));
    }
}

// ---------- processus enfants ----------
fn running(child: &mut Option<Child>) -> bool {
    matches!(child.as_mut().map(|c| c.try_wait()), Some(Ok(None)))
}

fn focus_gallery() {
    let t: Vec<u16> = "Shirokap".encode_utf16().chain([0]).collect();
    unsafe {
        let h = FindWindowW(std::ptr::null(), t.as_ptr());
        if !h.is_null() {
            ShowWindow(h, SW_RESTORE);
            SetForegroundWindow(h);
            InvalidateRect(h, std::ptr::null(), 0); // la fait se redessiner (et lire une éventuelle demande)
        }
    }
}

fn open_gallery(child: &mut Option<Child>, settings: bool) {
    if running(child) {
        if settings {
            // la galerie déjà ouverte passe sur les paramètres en lisant ce fichier
            let _ = std::fs::write(gallery_cmd_path(), "settings");
        }
        focus_gallery();
        return;
    }
    let args: &[&str] = if settings { &["--gui", "--settings"] } else { &["--gui"] };
    *child = spawn_self(args);
}

fn open_region(cfg: &Config, child: &mut Option<Child>, clip: bool) {
    if running(child) {
        return;
    }
    let title = active_title();
    if is_blocked(&cfg.blocklist, &title) {
        return;
    }
    let t = clean(&title, 40);
    *child = spawn_self(&[if clip { "--region-clip" } else { "--region" }, t.as_str()]);
}

/// Ouvre le menu moderne (un petit processus avec une fenêtre dessinée aux couleurs de l'appli).
fn open_popup(child: &mut Option<Child>, x: i32, y: i32, paused: bool, auto: bool) {
    if running(child) {
        return;
    }
    let _ = std::fs::remove_file(popup_cmd_path()); // pas de vieille commande oubliée
    let (xs, ys) = (x.to_string(), y.to_string());
    *child = spawn_self(&["--menu", xs.as_str(), ys.as_str(), if paused { "1" } else { "0" }, if auto { "1" } else { "0" }]);
    if let Some(c) = child {
        unsafe { AllowSetForegroundWindow(c.id()) }; // lui permet de prendre le focus
    }
}

/// Chemin complet de l'exécutable d'un processus (en minuscules), ou None si inaccessible.
fn exe_of(pid: u32) -> Option<String> {
    unsafe {
        let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if h.is_null() {
            return None;
        }
        let mut buf = [0u16; 1024];
        let mut n = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(h, PROCESS_NAME_WIN32, buf.as_mut_ptr(), &mut n);
        CloseHandle(h);
        if ok == 0 {
            return None;
        }
        Some(String::from_utf16_lossy(&buf[..n as usize]).to_lowercase())
    }
}

/// Fenêtres visibles des autres processus Shirokap (galerie, zone, captures épinglées…).
struct Scan {
    me: String,
    my_pid: u32,
    ours: HashMap<u32, bool>,
    wins: Vec<*mut c_void>,
}

unsafe extern "system" fn scan_window(h: *mut c_void, lp: isize) -> i32 {
    let s = &mut *(lp as *mut Scan);
    let mut pid = 0u32;
    GetWindowThreadProcessId(h, &mut pid);
    if pid != 0 && pid != s.my_pid && IsWindowVisible(h) != 0 {
        let me = s.me.clone();
        let ours = *s.ours.entry(pid).or_insert_with(|| exe_of(pid).as_deref() == Some(me.as_str()));
        if ours {
            s.wins.push(h);
        }
    }
    1 // continuer l'énumération
}

/// « Quitter » ferme toute l'appli : on demande poliment aux fenêtres de se fermer (comme le bouton ✕,
/// la galerie enregistre donc son état), puis on arrête au bout de 2 s celles qu'on a lancées et qui traînent.
fn close_app_windows(children: &mut [&mut Option<Child>]) {
    let me = std::env::current_exe().map(|p| p.display().to_string().to_lowercase()).unwrap_or_default();
    if !me.is_empty() {
        let mut s = Scan { me, my_pid: std::process::id(), ours: HashMap::new(), wins: Vec::new() };
        unsafe {
            EnumWindows(Some(scan_window), &mut s as *mut Scan as isize);
            for &h in &s.wins {
                PostMessageW(h, WM_CLOSE, 0, 0);
            }
        }
    }
    let deadline = Instant::now() + Duration::from_secs(2);
    for c in children.iter_mut() {
        while running(c) && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(50));
        }
        if running(c) {
            if let Some(ch) = c.as_mut() {
                let _ = ch.kill();
                log("a window did not close in time when quitting: process stopped");
            }
        }
        **c = None;
    }
}

fn parse_cmd(t: &str) -> Option<Cmd> {
    Some(match t.trim() {
        "gallery" => Cmd::Gallery,
        "settings" => Cmd::Settings,
        "snap" => Cmd::Snap(Mode::Screen),
        "snap_last" => Cmd::Snap(Mode::LastZone),
        "delayed_screen" => Cmd::Delayed(Mode::Screen),
        "delayed_window" => Cmd::Delayed(Mode::Window),
        "zone" => Cmd::Zone,
        "zone_clip" => Cmd::ZoneClip,
        "pick" => Cmd::Pick,
        "pin_last" => Cmd::PinLast,
        "trash_last" => Cmd::TrashLast,
        "auto" => Cmd::Auto,
        "last" => Cmd::Last,
        "copy_last" => Cmd::CopyLast,
        "folder" => Cmd::Folder,
        "config_dir" => Cmd::ConfigDir,
        "pause" => Cmd::Pause,
        "quit" => Cmd::Quit,
        "tog_clip" => Cmd::TogClip,
        "tog_sound" => Cmd::TogSound,
        "tog_flash" => Cmd::TogFlash,
        "tog_folder" => Cmd::TogFolder,
        s if s.starts_with("res:") => Cmd::Res(s[4..].parse().ok()?),
        _ => return None,
    })
}

pub fn run() {
    if already_running() {
        return;
    }
    let mut cfg = Config::load();
    if !config_path().exists() {
        cfg.save();
    }
    let mut mtime = config_mtime();
    // le processus du tray ne fait presque rien en attendant un raccourci : mode efficacité de Windows 11
    set_efficiency(true);
    if autostart_enabled() {
        set_autostart(true); // remet à jour le chemin si l'exe a été déplacé ou recompilé
    }

    let event_loop = EventLoop::new();
    let mgr = GlobalHotKeyManager::new().unwrap();
    let mut keys: Keys = [None; 7];
    rebind(&mgr, &mut keys, &cfg);

    // ----- menu -----
    let mut cmds: HashMap<MenuId, Cmd> = HashMap::new();
    let res: Vec<(u32, CheckMenuItem)> =
        RES.iter().map(|(w, label)| (*w, ck(&mut cmds, tr(*label), false, Cmd::Res(*w)))).collect();
    let it = Items {
        header: MenuItem::new("Shirokap", false, None),
        today: MenuItem::new("", false, None),
        snap: mi(&mut cmds, tr("Écran entier"), Cmd::Snap(Mode::Screen)),
        zone: mi(&mut cmds, tr("Zone…"), Cmd::Zone),
        zoneclip: mi(&mut cmds, tr("Zone → presse-papiers"), Cmd::ZoneClip),
        pick: mi(&mut cmds, tr("Pipette"), Cmd::Pick),
        lastzone: mi(&mut cmds, tr("Refaire la dernière zone"), Cmd::Snap(Mode::LastZone)),
        gallery: mi(&mut cmds, tr("Ouvrir la galerie"), Cmd::Gallery),
        auto: ck(&mut cmds, tr("Capture automatique (timelapse)"), false, Cmd::Auto),
        pause: ck(&mut cmds, tr("Suspendre les raccourcis"), false, Cmd::Pause),
        clip: ck(&mut cmds, tr("Copier dans le presse-papiers"), false, Cmd::TogClip),
        sound: ck(&mut cmds, tr("Bip sonore"), false, Cmd::TogSound),
        flash: ck(&mut cmds, tr("Clignotement de l'icône"), false, Cmd::TogFlash),
        folder: ck(&mut cmds, tr("Un dossier par fenêtre / jeu"), false, Cmd::TogFolder),
        res,
    };
    let mut today = today_count(&cfg);
    sync(&it, &cfg, today);

    let d_screen = mi(&mut cmds, tr("Écran dans 5 secondes"), Cmd::Delayed(Mode::Screen));
    let d_window = mi(&mut cmds, tr("Fenêtre active dans 5 secondes"), Cmd::Delayed(Mode::Window));
    let capture = Submenu::new(tr("Capturer"), true);
    capture
        .append_items(&[
            &it.snap,
            &it.zone,
            &it.zoneclip,
            &it.lastzone,
            &PredefinedMenuItem::separator(),
            &d_screen,
            &d_window,
            &PredefinedMenuItem::separator(),
            &it.pick,
        ])
        .unwrap();

    let r_open = mi(&mut cmds, tr("Ouvrir"), Cmd::Last);
    let r_copy = mi(&mut cmds, tr("Copier dans le presse-papiers"), Cmd::CopyLast);
    let recent = Submenu::new(tr("Dernière capture"), true);
    let r_pin = mi(&mut cmds, tr("Épingler à l'écran"), Cmd::PinLast);
    let r_trash = mi(&mut cmds, tr("Mettre à la corbeille"), Cmd::TrashLast);
    recent.append_items(&[&r_open, &r_copy, &r_pin, &PredefinedMenuItem::separator(), &r_trash]).unwrap();

    let f_caps = mi(&mut cmds, tr("Dossier des captures"), Cmd::Folder);
    let f_conf = mi(&mut cmds, tr("Dossier de configuration"), Cmd::ConfigDir);
    let dirs = Submenu::new(tr("Dossiers"), true);
    dirs.append_items(&[&f_caps, &f_conf]).unwrap();

    let quick = Submenu::new(tr("Réglages rapides"), true);
    quick
        .append_items(&[&it.clip, &it.sound, &it.flash, &it.folder, &PredefinedMenuItem::separator()])
        .unwrap();
    for (_, c) in &it.res {
        quick.append(c).unwrap();
    }

    let settings = mi(&mut cmds, tr("Paramètres…"), Cmd::Settings);
    let quit = mi(&mut cmds, tr("Quitter"), Cmd::Quit);
    // textes fixes du menu natif, remis à jour quand la langue change
    let mut statics: Vec<(Lbl, &'static str)> = vec![
        (Lbl::C(it.auto.clone()), "Capture automatique (timelapse)"),
        (Lbl::C(it.pause.clone()), "Suspendre les raccourcis"),
        (Lbl::C(it.clip.clone()), "Copier dans le presse-papiers"),
        (Lbl::C(it.sound.clone()), "Bip sonore"),
        (Lbl::C(it.flash.clone()), "Clignotement de l'icône"),
        (Lbl::C(it.folder.clone()), "Un dossier par fenêtre / jeu"),
        (Lbl::M(d_screen.clone()), "Écran dans 5 secondes"),
        (Lbl::M(d_window.clone()), "Fenêtre active dans 5 secondes"),
        (Lbl::S(capture.clone()), "Capturer"),
        (Lbl::M(r_open.clone()), "Ouvrir"),
        (Lbl::M(r_copy.clone()), "Copier dans le presse-papiers"),
        (Lbl::S(recent.clone()), "Dernière capture"),
        (Lbl::M(r_pin.clone()), "Épingler à l'écran"),
        (Lbl::M(r_trash.clone()), "Mettre à la corbeille"),
        (Lbl::M(f_caps.clone()), "Dossier des captures"),
        (Lbl::M(f_conf.clone()), "Dossier de configuration"),
        (Lbl::S(dirs.clone()), "Dossiers"),
        (Lbl::S(quick.clone()), "Réglages rapides"),
        (Lbl::M(settings.clone()), "Paramètres…"),
        (Lbl::M(quit.clone()), "Quitter"),
    ];
    for (w, c) in &it.res {
        if let Some((_, label)) = RES.iter().find(|r| r.0 == *w) {
            statics.push((Lbl::C(c.clone()), *label));
        }
    }
    let menu = Menu::new();
    menu.append_items(&[
        &it.header,
        &it.today,
        &PredefinedMenuItem::separator(),
        &capture,
        &it.auto,
        &PredefinedMenuItem::separator(),
        &it.gallery,
        &recent,
        &dirs,
        &PredefinedMenuItem::separator(),
        &quick,
        &it.pause,
        &settings,
        &PredefinedMenuItem::separator(),
        &quit,
    ])
    .unwrap();

    // Au démarrage de Windows, la barre des tâches n'est parfois pas encore prête : on réessaie (jusqu'à ~3 min)
    let mut built = None;
    for _ in 0..90 {
        let mut b = TrayIconBuilder::new()
            .with_menu_on_left_click(false)
            .with_icon(icon(cfg.accent_rgb()))
            .with_tooltip(tooltip(&cfg, &keys, false, false));
        if cfg.tray_menu == "native" {
            b = b.with_menu(Box::new(menu.clone())); // menu natif de Windows
        }
        let r = b.build();
        if let Ok(t) = r {
            built = Some(t);
            break;
        }
        std::thread::sleep(Duration::from_secs(2));
    }
    let Some(tray) = built else {
        log("could not create the tray icon (taskbar unavailable)");
        return;
    };
    log("tray icon created, daemon ready");

    let (tx, rx) = mpsc::channel::<bool>();
    let mut paused = false;
    let mut auto_on = false;
    let mut auto_next: Option<Instant> = None;
    let mut gallery: Option<Child> = None;
    let mut region: Option<Child> = None;
    let mut popup: Option<Child> = None;
    let mut native_menu = cfg.tray_menu == "native";
    let mut pending: Option<(Instant, Mode)> = None;
    let mut pick_at: Option<Instant> = None;
    let mut flash_until: Option<Instant> = None;
    let mut next_cleanup = Instant::now();

    event_loop.run(move |_, _, flow| {
        let now = Instant::now();
        // le démon dort entre deux réveils (~10 par seconde) : CPU ≈ 0 %
        *flow = ControlFlow::WaitUntil(now + Duration::from_millis(100));

        while let Ok(e) = GlobalHotKeyEvent::receiver().try_recv() {
            if paused || e.state() != HotKeyState::Pressed {
                continue;
            }
            match keys.iter().position(|k| k.map(|h| h.id()) == Some(e.id())) {
                Some(0) => trigger(&cfg, Mode::Screen, &tx, &mut pending),
                Some(1) => open_region(&cfg, &mut region, false),
                Some(2) => trigger(&cfg, Mode::Window, &tx, &mut pending),
                Some(3) => trigger(&cfg, Mode::LastZone, &tx, &mut pending),
                Some(4) => open_gallery(&mut gallery, false),
                Some(5) => open_region(&cfg, &mut region, true),
                Some(6) => {
                    if do_pick(&it) && cfg.flash {
                        let _ = tray.set_icon(Some(icon(GREEN)));
                        flash_until = Some(now + Duration::from_millis(600));
                    }
                }
                _ => {}
            }
        }
        while let Ok(e) = TrayIconEvent::receiver().try_recv() {
            if let TrayIconEvent::Click { button, button_state: MouseButtonState::Up, position, .. } = e {
                match button {
                    MouseButton::Left => open_gallery(&mut gallery, false),
                    MouseButton::Right => {
                        if cfg.tray_menu != "native" {
                            open_popup(&mut popup, position.x as i32, position.y as i32, paused, auto_on);
                        }
                    }
                    MouseButton::Middle => {
                        if !paused {
                            trigger(&cfg, Mode::Screen, &tx, &mut pending); // clic molette = capture
                        }
                    }
                }
            }
        }
        let mut todo: Vec<Cmd> = Vec::new();
        while let Ok(e) = MenuEvent::receiver().try_recv() {
            if let Some(c) = cmds.get(&e.id).copied() {
                todo.push(c);
            }
        }
        // résultat du menu moderne : il se ferme, puis dépose la commande choisie dans un fichier
        if popup.as_mut().map(|c| matches!(c.try_wait(), Ok(Some(_)))).unwrap_or(false) {
            popup = None;
            if let Ok(t) = std::fs::read_to_string(popup_cmd_path()) {
                let _ = std::fs::remove_file(popup_cmd_path());
                if let Some(c) = parse_cmd(&t) {
                    todo.push(c);
                }
            }
        }
        for cmd in todo {
            match cmd {
                Cmd::Gallery => open_gallery(&mut gallery, false),
                Cmd::Settings => open_gallery(&mut gallery, true),
                Cmd::Snap(m) => {
                    std::thread::sleep(Duration::from_millis(400)); // laisse le menu se fermer
                    shoot(&cfg, m, &tx, false);
                }
                Cmd::Delayed(m) => pending = Some((now + Duration::from_secs(5), m)),
                Cmd::Zone => open_region(&cfg, &mut region, false),
                Cmd::ZoneClip => open_region(&cfg, &mut region, true),
                Cmd::Pick => pick_at = Some(now + Duration::from_secs(3)),
                Cmd::PinLast => {
                    if let Some(i) = list_items(&cfg.save_dir).first() {
                        let s = i.path.display().to_string();
                        let _ = spawn_self(&["--pin", s.as_str()]);
                    }
                }
                Cmd::TrashLast => {
                    if let Some(i) = list_items(&cfg.save_dir).first() {
                        if trash::delete(&i.path).is_ok() {
                            let _ = std::fs::remove_file(thumb_path(&i.path));
                        }
                    }
                    today = today_count(&cfg);
                    sync(&it, &cfg, today);
                }
                Cmd::Auto => {
                    auto_on = !auto_on;
                    it.auto.set_checked(auto_on);
                    auto_next = if auto_on { Some(now + Duration::from_secs(cfg.interval_secs.max(1) as u64)) } else { None };
                    let _ = tray.set_tooltip(Some(tooltip(&cfg, &keys, paused, auto_on)));
                }
                Cmd::Last => {
                    if let Some(i) = list_items(&cfg.save_dir).first() {
                        let _ = Command::new("explorer").arg(&i.path).spawn();
                    }
                }
                Cmd::CopyLast => copy_last(&cfg),
                Cmd::Folder => {
                    let _ = Command::new("explorer").arg(&cfg.save_dir).spawn();
                }
                Cmd::ConfigDir => {
                    if let Some(d) = config_path().parent() {
                        let _ = Command::new("explorer").arg(d).spawn();
                    }
                }
                Cmd::Pause => {
                    paused = !paused;
                    it.pause.set_checked(paused);
                    let _ = tray.set_icon(Some(icon(if paused { GRAY } else { cfg.accent_rgb() })));
                    let _ = tray.set_tooltip(Some(tooltip(&cfg, &keys, paused, auto_on)));
                }
                Cmd::Quit => {
                    close_app_windows(&mut [&mut gallery, &mut region, &mut popup]);
                    log("quit from the tray menu");
                    *flow = ControlFlow::Exit;
                }
                Cmd::TogClip | Cmd::TogSound | Cmd::TogFlash | Cmd::TogFolder | Cmd::Res(_) => {
                    match cmd {
                        Cmd::TogClip => cfg.clipboard = !cfg.clipboard,
                        Cmd::TogSound => cfg.sound = !cfg.sound,
                        Cmd::TogFlash => cfg.flash = !cfg.flash,
                        Cmd::TogFolder => cfg.folder_per_window = !cfg.folder_per_window,
                        Cmd::Res(w) => cfg.target_width = w,
                        _ => {}
                    }
                    persist(&cfg, &mut mtime);
                    sync(&it, &cfg, today);
                }
            }
        }

        // minuteur (délai avant capture)
        if let Some((t, m)) = pending {
            if now >= t {
                pending = None;
                shoot(&cfg, m, &tx, false);
            }
        }
        // pipette différée (depuis le menu)
        if let Some(t) = pick_at {
            if now >= t {
                pick_at = None;
                if do_pick(&it) && cfg.flash {
                    let _ = tray.set_icon(Some(icon(GREEN)));
                    flash_until = Some(now + Duration::from_millis(600));
                }
            }
        }
        // timelapse
        if let Some(t) = auto_next {
            if auto_on && now >= t {
                auto_next = Some(now + Duration::from_secs(cfg.interval_secs.max(1) as u64));
                if !paused {
                    shoot(&auto_cfg(&cfg), Mode::Screen, &tx, true);
                }
            }
        }
        // capture terminée : compteur du menu + clignotement vert
        while let Ok(flash) = rx.try_recv() {
            today = today_count(&cfg);
            sync(&it, &cfg, today);
            if flash && cfg.flash && !paused {
                let _ = tray.set_icon(Some(icon(GREEN)));
                flash_until = Some(now + Duration::from_millis(600));
            }
        }
        if let Some(t) = flash_until {
            if now >= t {
                flash_until = None;
                let _ = tray.set_icon(Some(icon(if paused { GRAY } else { cfg.accent_rgb() })));
            }
        }
        // nettoyage automatique des vieilles captures (si activé), toutes les 6 h
        if now >= next_cleanup {
            next_cleanup = now + Duration::from_secs(6 * 3600);
            let c = cfg.clone();
            std::thread::spawn(move || cleanup_old(&c));
        }
        // rechargement à chaud de la config
        let m = config_mtime();
        if m != mtime {
            mtime = m;
            cfg = Config::load();
            rebind(&mgr, &mut keys, &cfg);
            set_lang(&cfg.language);
            relabel(&statics);
            let now_native = cfg.tray_menu == "native";
            if now_native != native_menu {
                native_menu = now_native;
                let m: Option<Box<dyn ContextMenu>> = if now_native { Some(Box::new(menu.clone())) } else { None };
                tray.set_menu(m);
            }
            today = today_count(&cfg);
            sync(&it, &cfg, today);
            let _ = tray.set_icon(Some(icon(if paused { GRAY } else { cfg.accent_rgb() })));
            let _ = tray.set_tooltip(Some(tooltip(&cfg, &keys, paused, auto_on)));
        }
    });
}
