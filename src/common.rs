use image::{
    codecs::{
        jpeg::JpegEncoder,
        png::{CompressionType, FilterType as PngFilter, PngEncoder},
    },
    imageops::FilterType,
    ExtendedColorType, ImageEncoder, RgbaImage,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map::DefaultHasher, HashSet},
    fs,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    process::{Child, Command},
    time::SystemTime,
};
use windows_sys::Win32::{
    Foundation::POINT,
    Media::Audio::{PlaySoundW, SND_MEMORY},
    UI::{
        HiDpi::{SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2},
        WindowsAndMessaging::GetCursorPos,
    },
};

static BEEP: &[u8] = include_bytes!("../assets/beep.wav");

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Config {
    pub hotkey: String,         // capture d'écran
    pub hotkey_region: String,  // capture de zone
    pub hotkey_window: String,  // capture de la fenêtre active
    pub hotkey_last_region: String, // refaire la dernière zone
    pub hotkey_gallery: String, // ouvrir la galerie
    pub hotkey_region_clip: String, // zone vers le presse-papiers (sans fichier)
    pub hotkey_picker: String,  // pipette : copie la couleur sous le curseur
    pub interval_secs: u32,     // intervalle du timelapse
    pub blocklist: String,      // fenêtres à ne jamais capturer (mots séparés par des virgules)
    pub cleanup_days: u32,      // 0 = jamais ; sinon corbeille des captures plus vieilles (favoris épargnés)
    pub editor: String,         // éditeur externe (vide = Paint)
    pub monitor_mode: String,   // "primary" | "cursor" | "all"
    pub target_width: u32,      // 0 = résolution native, sinon largeur de sortie (3840 = 4K)
    pub delay_secs: u32,
    pub save_dir: String,
    pub name_template: String,  // {window} {date} {time} {res} {n}
    pub format: String,         // "png" | "jpg"
    pub jpg_quality: u8,
    pub png_level: u8,          // 0 rapide, 1 normal, 2 maximal
    pub folder_per_window: bool,
    pub clipboard: bool,
    pub sound: bool,
    pub flash: bool,
    pub theme: String,          // "dark" | "light"
    pub accent: [u8; 3],        // couleur d'accent fixe (utilisée quand accent_auto est faux)
    #[serde(default)]           // ancienne config sans ce réglage : on garde la couleur fixe choisie
    pub accent_auto: bool,      // vrai : l'accent suit le thème (couleur propre à chaque thème)
    pub tray_menu: String,      // "modern" (menu dessiné par l'appli) | "native" (menu Windows)
    pub language: String,       // "auto" (langue de Windows) | "fr" | "en" | "es" | "de" | "zh"
}

/// Mode économie d'énergie de Windows 11 (« EcoQoS ») : quand il est actif, Windows fait tourner le
/// processus sur des cœurs moins rapides et plus économes. La feuille « Mode efficacité » du gestionnaire
/// des tâches n'apparaît pas : elle demande en plus une priorité basse, qu'on évite pour que le tray
/// réagisse tout de suite aux raccourcis même quand un jeu occupe le processeur.
/// Idéal pour ce qui travaille en arrière-plan ; sans effet (et sans erreur) sur les anciennes versions de Windows.
pub fn set_efficiency(on: bool) {
    use windows_sys::Win32::System::Threading::{
        GetCurrentProcess, ProcessPowerThrottling, SetProcessInformation, PROCESS_POWER_THROTTLING_CURRENT_VERSION,
        PROCESS_POWER_THROTTLING_EXECUTION_SPEED, PROCESS_POWER_THROTTLING_STATE,
    };
    let state = PROCESS_POWER_THROTTLING_STATE {
        Version: PROCESS_POWER_THROTTLING_CURRENT_VERSION,
        ControlMask: PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
        StateMask: if on { PROCESS_POWER_THROTTLING_EXECUTION_SPEED } else { 0 },
    };
    unsafe {
        SetProcessInformation(
            GetCurrentProcess(),
            ProcessPowerThrottling,
            &state as *const _ as *const std::ffi::c_void,
            std::mem::size_of::<PROCESS_POWER_THROTTLING_STATE>() as u32,
        );
    }
}

/// Couleur d'accent propre à chaque thème (mode « Auto »).
pub fn theme_accent(theme: &str) -> [u8; 3] {
    match theme {
        "shirokap" => [255, 255, 255], // blanc, comme le losange du logo
        "graphite" => [20, 184, 166],
        "midnight" => [99, 132, 255],
        "oled" => [34, 211, 238],
        "ocean" => [45, 212, 191],
        "forest" => [74, 222, 128],
        "dusk" => [192, 132, 252],
        "ruby" => [244, 63, 94],
        "glacier" => [125, 196, 228],
        "mocha" => [217, 160, 102],
        "neon" => [217, 70, 239],
        "light" => [37, 99, 235],
        "sand" => [194, 120, 62],
        "mint" => [16, 163, 127],
        "lavender" => [124, 58, 237],
        "sakura" => [219, 39, 119],
        "sky" => [2, 132, 199],
        _ => [59, 130, 246], // sombre
    }
}

impl Config {
    /// Couleur d'accent réellement affichée : celle du thème en mode « Auto », sinon la couleur fixe.
    pub fn accent_rgb(&self) -> [u8; 3] {
        if self.accent_auto {
            theme_accent(&self.theme)
        } else {
            self.accent
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        let dir = directories::UserDirs::new()
            .and_then(|d| d.picture_dir().map(|p| p.join("Shirokap")))
            .unwrap_or_else(|| PathBuf::from("Shirokap"));
        Self {
            hotkey: "F12".into(),
            hotkey_region: "Shift+F12".into(),
            hotkey_window: "Alt+F12".into(),
            hotkey_last_region: "Ctrl+F12".into(),
            hotkey_gallery: "Ctrl+Shift+F12".into(),
            hotkey_region_clip: "Shift+F11".into(),
            hotkey_picker: "Alt+F11".into(),
            interval_secs: 30,
            blocklist: String::new(),
            cleanup_days: 0,
            editor: "mspaint".into(),
            monitor_mode: "primary".into(),
            target_width: 0,
            delay_secs: 0,
            save_dir: dir.to_string_lossy().into(),
            name_template: "{date}_{time}".into(),
            format: "png".into(),
            jpg_quality: 90,
            png_level: 0,
            folder_per_window: false,
            clipboard: true,
            sound: false,
            flash: true,
            theme: "dark".into(),
            accent: [59, 130, 246],
            accent_auto: true,
            tray_menu: "modern".into(),
            language: "auto".into(),
        }
    }
}

fn proj() -> directories::ProjectDirs {
    directories::ProjectDirs::from("", "", "Shirokap").expect("dossier utilisateur introuvable")
}
pub fn config_path() -> PathBuf {
    proj().config_dir().join("config.toml")
}
/// Fichier où le menu moderne dépose la commande choisie (lue ensuite par le démon).
pub fn popup_cmd_path() -> PathBuf {
    proj().data_local_dir().join("popup_cmd.txt")
}
pub fn config_mtime() -> Option<SystemTime> {
    fs::metadata(config_path()).and_then(|m| m.modified()).ok()
}
impl Config {
    pub fn load() -> Self {
        match fs::read_to_string(config_path()) {
            Ok(s) => match toml::from_str(&s) {
                Ok(c) => c,
                Err(e) => {
                    log(&format!("config.toml unreadable, default settings used: {e}"));
                    let _ = fs::copy(config_path(), config_path().with_extension("toml.bad")); // copie de secours
                    Config::default()
                }
            },
            Err(_) => Config::default(),
        }
    }
    pub fn save(&self) {
        let p = config_path();
        if let Some(d) = p.parent() {
            let _ = fs::create_dir_all(d);
        }
        if let Ok(s) = toml::to_string_pretty(self) {
            let _ = fs::write(p, s);
        }
    }
}

// ---------- système ----------
pub fn dpi_aware() {
    unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };
}
pub fn spawn_self(args: &[&str]) -> Option<Child> {
    match Command::new(std::env::current_exe().ok()?).args(args).spawn() {
        Ok(c) => Some(c),
        Err(e) => {
            log(&format!("could not start ({args:?}): {e}"));
            None
        }
    }
}

// ---------- journal ----------
pub fn log_dir() -> PathBuf {
    proj().data_local_dir().join("logs")
}
pub fn log_path() -> PathBuf {
    log_dir().join("shirokap.log")
}

/// Ajoute une ligne au journal (%LOCALAPPDATA%\Shirokap\data\logs). Ne bloque jamais : toute erreur d'écriture est ignorée.
pub fn log(msg: &str) {
    use std::io::Write;
    let dir = log_dir();
    let _ = fs::create_dir_all(&dir);
    let path = log_path();
    if fs::metadata(&path).map(|m| m.len() > 512 * 1024).unwrap_or(false) {
        let _ = fs::rename(&path, dir.join("shirokap.old.log")); // rotation : on garde l'ancien journal
    }
    if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(&path) {
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let _ = writeln!(f, "[{now}] [pid {}] {msg}", std::process::id());
    }
}

/// Un plantage (panic) est écrit dans le journal avant la fermeture de l'appli.
pub fn install_panic_hook(mode: &'static str) {
    std::panic::set_hook(Box::new(move |info| {
        log(&format!("CRASH ({mode}, v{}): {info}", env!("CARGO_PKG_VERSION")));
    }));
}
/// Capture l'écran qui contient le curseur. Renvoie aussi la position (x, y) de cet écran.
pub fn monitor_shot_at_cursor() -> Option<(RgbaImage, i32, i32)> {
    let mut p = POINT { x: 0, y: 0 };
    unsafe { GetCursorPos(&mut p) };
    let m = xcap::Monitor::from_point(p.x, p.y).ok()?;
    let (mx, my) = (m.x().ok()?, m.y().ok()?);
    Some((opaque(m.capture_image().ok()?), mx, my))
}
pub fn opaque(mut img: RgbaImage) -> RgbaImage {
    for p in img.pixels_mut() {
        p.0[3] = 255;
    }
    img
}
pub fn clean(s: &str, max: usize) -> String {
    let t: String = s
        .chars()
        .map(|c| if "<>:\"/\\|?*".contains(c) || c.is_control() { '_' } else { c })
        .take(max)
        .collect();
    t.trim_matches(|c| c == ' ' || c == '.').to_string()
}
pub fn human(b: u64) -> String {
    let b = b as f64;
    let [ko, mo, go] = crate::i18n::size_units();
    if b >= 1e9 {
        format!("{:.1} {go}", b / 1e9)
    } else if b >= 1e6 {
        format!("{:.1} {mo}", b / 1e6)
    } else {
        format!("{:.0} {ko}", b / 1e3)
    }
}

// ---------- enregistrement ----------
fn rgb_bytes(img: &RgbaImage) -> Vec<u8> {
    let mut v = Vec::with_capacity(img.width() as usize * img.height() as usize * 3);
    for p in img.pixels() {
        v.extend_from_slice(&p.0[..3]);
    }
    v
}

pub fn save_jpeg(img: &RgbaImage, path: &Path, quality: u8) -> Option<()> {
    if let Some(d) = path.parent() {
        fs::create_dir_all(d).ok()?;
    }
    let f = std::io::BufWriter::new(fs::File::create(path).ok()?);
    JpegEncoder::new_with_quality(f, quality)
        .write_image(&rgb_bytes(img), img.width(), img.height(), ExtendedColorType::Rgb8)
        .ok()
}

pub fn save_png(img: &RgbaImage, path: &Path, level: u8) -> Option<()> {
    let f = std::io::BufWriter::new(fs::File::create(path).ok()?);
    let c = match level {
        0 => CompressionType::Fast,
        1 => CompressionType::Default,
        _ => CompressionType::Best,
    };
    PngEncoder::new_with_quality(f, c, PngFilter::Adaptive)
        .write_image(&rgb_bytes(img), img.width(), img.height(), ExtendedColorType::Rgb8)
        .ok()
}

pub fn thumb_path(img: &Path) -> PathBuf {
    let mut h = DefaultHasher::new();
    img.hash(&mut h);
    proj().data_local_dir().join("thumbs").join(format!("{:x}.jpg", h.finish()))
}

pub fn make_thumb(img: &RgbaImage, dst: &Path) {
    let h = (480 * img.height() / img.width().max(1)).max(1);
    let t = image::imageops::thumbnail(img, 480, h);
    let _ = save_jpeg(&t, dst, 80);
}

pub fn copy_to_clipboard(img: &RgbaImage) {
    if let Ok(mut cb) = arboard::Clipboard::new() {
        let _ = cb.set_image(arboard::ImageData {
            width: img.width() as usize,
            height: img.height() as usize,
            bytes: std::borrow::Cow::Borrowed(img.as_raw().as_slice()),
        });
    }
}

/// Redimensionne (4K…), nomme, sauvegarde, crée la miniature, copie, bipe. Renvoie le chemin.
pub fn finish(cfg: &Config, mut img: RgbaImage, title: &str) -> Option<PathBuf> {
    if cfg.target_width > 0 && img.width() != cfg.target_width {
        let h = (img.height() as u64 * cfg.target_width as u64 / img.width().max(1) as u64).max(1) as u32;
        img = image::imageops::resize(&img, cfg.target_width, h, FilterType::Lanczos3);
    }
    let mut title = clean(title, 40);
    if title.is_empty() {
        title = "Bureau".into();
    }
    let mut dir = PathBuf::from(&cfg.save_dir);
    if cfg.folder_per_window {
        dir.push(&title);
    }
    let n_str = format!("{:04}", fs::read_dir(&dir).map(|d| d.flatten().count()).unwrap_or(0) + 1);
    let now = chrono::Local::now();
    let name = clean(
        &cfg.name_template
            .replace("{window}", &title)
            .replace("{date}", &now.format("%Y-%m-%d").to_string())
            .replace("{time}", &now.format("%Hh%Mm%Ss").to_string())
            .replace("{res}", &format!("{}x{}", img.width(), img.height()))
            .replace("{n}", &n_str),
        120,
    );
    let name = if name.is_empty() { "capture".to_string() } else { name };

    fs::create_dir_all(&dir).ok()?;
    let jpg = cfg.format == "jpg";
    let ext = if jpg { "jpg" } else { "png" };
    let mut path = dir.join(format!("{name}.{ext}"));
    let mut n = 2;
    while path.exists() {
        path = dir.join(format!("{name}_{n}.{ext}"));
        n += 1;
    }
    let ok = if jpg { save_jpeg(&img, &path, cfg.jpg_quality) } else { save_png(&img, &path, cfg.png_level) };
    if ok.is_none() {
        log(&format!("failed to write file: {}", path.display()));
        return None;
    }
    make_thumb(&img, &thumb_path(&path));
    if cfg.clipboard {
        copy_to_clipboard(&img);
    }
    if cfg.sound {
        // assets/beep.wav est intégré dans l'exe ; on est dans un thread, donc lecture synchrone
        unsafe { PlaySoundW(BEEP.as_ptr() as *const u16, std::ptr::null_mut(), SND_MEMORY) };
    }
    Some(path)
}

// ---------- favoris ----------
fn fav_path() -> PathBuf {
    proj().config_dir().join("favorites.txt")
}
pub fn load_favs() -> HashSet<PathBuf> {
    fs::read_to_string(fav_path())
        .map(|s| s.lines().filter(|l| !l.is_empty()).map(PathBuf::from).collect())
        .unwrap_or_default()
}
pub fn save_favs(f: &HashSet<PathBuf>) {
    let txt: Vec<String> = f.iter().map(|p| p.display().to_string()).collect();
    if let Some(d) = fav_path().parent() {
        let _ = fs::create_dir_all(d);
    }
    let _ = fs::write(fav_path(), txt.join("\n"));
}

// ---------- liste des captures ----------
pub struct Item {
    pub path: PathBuf,
    pub folder: String,
    pub modified: SystemTime,
    pub size: u64,
}

fn scan(dir: &Path, folder: &str, out: &mut Vec<Item>) {
    if let Ok(rd) = fs::read_dir(dir) {
        for e in rd.flatten() {
            let p = e.path();
            let ext = p.extension().and_then(|x| x.to_str()).unwrap_or("").to_lowercase();
            if matches!(ext.as_str(), "png" | "jpg" | "jpeg") {
                if let Ok(m) = e.metadata() {
                    if let Ok(t) = m.modified() {
                        out.push(Item { path: p, folder: folder.to_string(), modified: t, size: m.len() });
                    }
                }
            }
        }
    }
}

pub fn list_items(root: &str) -> Vec<Item> {
    let root = Path::new(root);
    let mut out = Vec::new();
    scan(root, "", &mut out);
    if let Ok(rd) = fs::read_dir(root) {
        for e in rd.flatten() {
            if e.path().is_dir() {
                scan(&e.path(), &e.file_name().to_string_lossy(), &mut out);
            }
        }
    }
    out.sort_by(|a, b| b.modified.cmp(&a.modified));
    out
}

// ---------- lancement au démarrage (registre) ----------
const RUN_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";

fn hidden(cmd: &mut Command) -> bool {
    use std::os::windows::process::CommandExt;
    cmd.creation_flags(0x0800_0000).status().map(|s| s.success()).unwrap_or(false)
}
pub fn autostart_enabled() -> bool {
    hidden(Command::new("reg").args(["query", RUN_KEY, "/v", "Shirokap"]))
}
pub fn set_autostart(on: bool) {
    if on {
        // si l'entrée avait été désactivée dans le Gestionnaire des tâches, on la réactive
        hidden(Command::new("reg").args([
            "delete",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run",
            "/v",
            "Shirokap",
            "/f",
        ]));
        let exe = std::env::current_exe().map(|p| p.display().to_string()).unwrap_or_default();
        let val = format!("\"{exe}\"");
        hidden(Command::new("reg").args(["add", RUN_KEY, "/v", "Shirokap", "/t", "REG_SZ", "/d", val.as_str(), "/f"]));
    } else {
        hidden(Command::new("reg").args(["delete", RUN_KEY, "/v", "Shirokap", "/f"]));
    }
}

// ---------- dernière zone, blocklist, rangement, presse-papiers ----------
fn last_region_path() -> PathBuf {
    proj().config_dir().join("last_region.txt")
}
/// (x, y, largeur, hauteur) dans l'écran, puis position (mx, my) de cet écran.
pub fn save_last_region(x: u32, y: u32, w: u32, h: u32, mx: i32, my: i32) {
    if let Some(d) = last_region_path().parent() {
        let _ = fs::create_dir_all(d);
    }
    let _ = fs::write(last_region_path(), format!("{x} {y} {w} {h} {mx} {my}"));
}
pub fn load_last_region() -> Option<(u32, u32, u32, u32, i32, i32)> {
    let s = fs::read_to_string(last_region_path()).ok()?;
    let v: Vec<i64> = s.split_whitespace().filter_map(|x| x.parse().ok()).collect();
    if v.len() >= 4 && v[..4].iter().all(|n| *n >= 0) {
        let (mx, my) = if v.len() >= 6 { (v[4] as i32, v[5] as i32) } else { (0, 0) };
        Some((v[0] as u32, v[1] as u32, v[2] as u32, v[3] as u32, mx, my))
    } else {
        None
    }
}

pub fn is_blocked(list: &str, title: &str) -> bool {
    let t = title.to_lowercase();
    list.split(|c| c == ',' || c == ';')
        .map(|s| s.trim().to_lowercase())
        .any(|s| !s.is_empty() && t.contains(&s))
}

pub fn copy_text(s: &str) {
    if let Ok(mut cb) = arboard::Clipboard::new() {
        let _ = cb.set_text(s.to_string());
    }
}

pub fn copy_last(cfg: &Config) {
    if let Some(it) = list_items(&cfg.save_dir).first() {
        if let Ok(i) = image::open(&it.path) {
            copy_to_clipboard(&i.to_rgba8());
        }
    }
}

pub fn today_count(cfg: &Config) -> usize {
    let today = chrono::Local::now().date_naive();
    list_items(&cfg.save_dir)
        .iter()
        .filter(|i| chrono::DateTime::<chrono::Local>::from(i.modified).date_naive() == today)
        .count()
}

/// Envoie à la corbeille les captures plus vieilles que `cleanup_days` (les favoris sont épargnés).
pub fn cleanup_old(cfg: &Config) {
    if cfg.cleanup_days == 0 {
        return;
    }
    let limit = std::time::Duration::from_secs(cfg.cleanup_days as u64 * 86_400);
    let favs = load_favs();
    let now = SystemTime::now();
    for it in list_items(&cfg.save_dir) {
        let old = now.duration_since(it.modified).map(|d| d > limit).unwrap_or(false);
        if old && !favs.contains(&it.path) && trash::delete(&it.path).is_ok() {
            let _ = fs::remove_file(thumb_path(&it.path));
        }
    }
}

// ---------- fichiers dans le presse-papiers, import, état de la galerie ----------
/// Place des FICHIERS dans le presse-papiers (Ctrl+V dans l'Explorateur, un mail…).
pub fn copy_files(paths: &[PathBuf]) {
    if let Ok(mut cb) = arboard::Clipboard::new() {
        let _ = cb.set().file_list(paths);
    }
}

/// Copie une image (glissée dans la galerie) dans le dossier de captures.
pub fn import_file(cfg: &Config, p: &Path) -> bool {
    let ext = p.extension().and_then(|x| x.to_str()).unwrap_or("").to_lowercase();
    if !matches!(ext.as_str(), "png" | "jpg" | "jpeg") {
        return false;
    }
    let Some(name) = p.file_stem().map(|s| s.to_string_lossy().to_string()) else { return false };
    let dir = PathBuf::from(&cfg.save_dir);
    if fs::create_dir_all(&dir).is_err() {
        return false;
    }
    let mut dest = dir.join(format!("{name}.{ext}"));
    let mut n = 2;
    while dest.exists() {
        dest = dir.join(format!("{name}_{n}.{ext}"));
        n += 1;
    }
    fs::copy(p, dest).is_ok()
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct UiState {
    pub thumb_w: f32,
    pub sort: u8, // 0 récentes, 1 anciennes, 2 nom, 3 taille
    pub favs_only: bool,
    pub sidebar: bool, // barre latérale ouverte ou réduite
    pub welcomed: bool, // écran d'accueil déjà vu
    pub slide_new_first: bool, // diaporama : du plus récent au plus ancien (sinon l'inverse)
}
impl Default for UiState {
    fn default() -> Self {
        Self { thumb_w: 250.0, sort: 0, favs_only: false, sidebar: true, welcomed: false, slide_new_first: false }
    }
}
fn ui_path() -> PathBuf {
    proj().config_dir().join("ui_state.toml")
}
pub fn load_ui() -> UiState {
    fs::read_to_string(ui_path()).ok().and_then(|s| toml::from_str(&s).ok()).unwrap_or_default()
}
pub fn save_ui(u: &UiState) {
    if let Some(d) = ui_path().parent() {
        let _ = fs::create_dir_all(d);
    }
    if let Ok(s) = toml::to_string(u) {
        let _ = fs::write(ui_path(), s);
    }
}
