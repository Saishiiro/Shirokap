//! Traductions de l'interface.
//!
//! Chaque texte est écrit en français dans le code et sert de clé : `tr("Ouvrir la galerie")`
//! renvoie sa version dans la langue active. Les traductions sont dans `i18n_table.rs`.
//! Langues : français et anglais (relus), espagnol, allemand et chinois (traductions non vérifiées).
//! Une clé absente du tableau s'affiche en anglais si possible, sinon en français.
use std::{
    collections::HashMap,
    fmt::Display,
    sync::{
        atomic::{AtomicU8, Ordering},
        OnceLock,
    },
};

pub const FR: u8 = 0;
pub const EN: u8 = 1;
pub const ES: u8 = 2;
pub const DE: u8 = 3;
pub const ZH: u8 = 4;

static LANG: AtomicU8 = AtomicU8::new(FR);

/// Réglage → (clé de config, nom affiché dans sa propre langue, traduction relue ?)
pub const LANGUAGES: [(&str, &str, bool); 6] = [
    ("auto", "", true),
    ("fr", "Français", true),
    ("en", "English", true),
    ("es", "Español", false),
    ("de", "Deutsch", false),
    ("zh", "中文（简体）", false),
];

/// Langue de l'interface de Windows (les langues non traduites passent en anglais).
fn system_lang() -> u8 {
    let id = unsafe { windows_sys::Win32::Globalization::GetUserDefaultUILanguage() } & 0x3ff;
    match id {
        0x0c => FR,
        0x0a => ES,
        0x07 => DE,
        0x04 => ZH,
        _ => EN,
    }
}

/// Active la langue d'un réglage (« auto », « fr », « en »…).
pub fn set_lang(code: &str) {
    let l = match code {
        "fr" => FR,
        "en" => EN,
        "es" => ES,
        "de" => DE,
        "zh" => ZH,
        _ => system_lang(),
    };
    LANG.store(l, Ordering::Relaxed);
}

pub fn lang() -> u8 {
    LANG.load(Ordering::Relaxed)
}

fn index() -> &'static HashMap<&'static str, usize> {
    static IDX: OnceLock<HashMap<&'static str, usize>> = OnceLock::new();
    IDX.get_or_init(|| crate::i18n_table::TABLE.iter().enumerate().map(|(i, row)| (row[0], i)).collect())
}

/// Partie affichée d'une clé : « Enregistrer|raccourci » s'affiche « Enregistrer » en français.
/// Le suffixe après « | » distingue deux textes français identiques qui se traduisent différemment.
fn visible(fr: &'static str) -> &'static str {
    match fr.find('|') {
        Some(i) => &fr[..i],
        None => fr,
    }
}

/// Texte traduit dans la langue active.
pub fn tr(fr: &'static str) -> &'static str {
    let l = lang() as usize;
    if l == 0 {
        return visible(fr);
    }
    match index().get(fr) {
        Some(&i) => {
            let row = &crate::i18n_table::TABLE[i];
            if !row[l].is_empty() {
                row[l]
            } else if !row[1].is_empty() {
                row[1]
            } else {
                visible(fr)
            }
        }
        None => visible(fr),
    }
}

/// Texte traduit avec des valeurs : `trf("{0} fichier(s) copié(s)", &[&n])`.
pub fn trf(fr: &'static str, args: &[&dyn Display]) -> String {
    let mut s = tr(fr).to_string();
    for (i, a) in args.iter().enumerate() {
        s = s.replace(&format!("{{{i}}}"), &a.to_string());
    }
    s
}

/// « 3 captures » / « 3 screenshots » / « 3 张截图 »…
pub fn n_captures(n: usize) -> String {
    match lang() {
        EN => format!("{n} {}", if n == 1 { "screenshot" } else { "screenshots" }),
        ES => format!("{n} {}", if n == 1 { "captura" } else { "capturas" }),
        DE => format!("{n} {}", if n == 1 { "Aufnahme" } else { "Aufnahmen" }),
        ZH => format!("{n} 张截图"),
        _ => format!("{n} {}", if n > 1 { "captures" } else { "capture" }),
    }
}

/// Ligne « N captures aujourd'hui » du menu du tray.
pub fn today_label(n: usize) -> String {
    if n == 0 {
        return tr("Aucune capture aujourd'hui").to_string();
    }
    match lang() {
        EN => format!("{} today", n_captures(n)),
        ES => format!("{} hoy", n_captures(n)),
        DE => format!("{} heute", n_captures(n)),
        ZH => format!("今天 {}", n_captures(n)),
        _ => format!("{} aujourd'hui", n_captures(n)),
    }
}

/// Format de date : `long` = jour, mois et année ; sinon jour et mois.
pub fn dfmt(long: bool) -> &'static str {
    match (lang(), long) {
        (EN, true) => "%m/%d/%Y",
        (EN, false) => "%m/%d",
        (DE, true) => "%d.%m.%Y",
        (DE, false) => "%d.%m.",
        (ZH, true) => "%Y/%m/%d",
        (ZH, false) => "%m/%d",
        (_, true) => "%d/%m/%Y",
        (_, false) => "%d/%m",
    }
}

/// Vrai si la langue écrit les nombres décimaux avec une virgule.
pub fn decimal_comma() -> bool {
    matches!(lang(), FR | ES | DE)
}

/// « N dossiers ».
pub fn n_folders(n: usize) -> String {
    match lang() {
        EN => format!("{n} {}", if n == 1 { "folder" } else { "folders" }),
        ES => format!("{n} {}", if n == 1 { "carpeta" } else { "carpetas" }),
        DE => format!("{n} Ordner"),
        ZH => format!("{n} 个文件夹"),
        _ => format!("{n} {}", if n > 1 { "dossiers" } else { "dossier" }),
    }
}

/// « N jours ».
pub fn n_days(n: usize) -> String {
    match lang() {
        EN => format!("{n} {}", if n == 1 { "day" } else { "days" }),
        ES => format!("{n} {}", if n == 1 { "día" } else { "días" }),
        DE => format!("{n} {}", if n == 1 { "Tag" } else { "Tage" }),
        ZH => format!("{n} 天"),
        _ => format!("{n} {}", if n > 1 { "jours" } else { "jour" }),
    }
}

/// Initiales des jours de la semaine, du lundi au dimanche.
pub fn weekday_letters() -> [&'static str; 7] {
    match lang() {
        EN => ["M", "T", "W", "T", "F", "S", "S"],
        ES => ["L", "M", "X", "J", "V", "S", "D"],
        DE => ["M", "D", "M", "D", "F", "S", "S"],
        ZH => ["一", "二", "三", "四", "五", "六", "日"],
        _ => ["L", "M", "M", "J", "V", "S", "D"],
    }
}

/// Heure de la journée : « 14 h » en français et en espagnol, « 14:00 » ailleurs.
pub fn hour_label(h: usize) -> String {
    match lang() {
        FR | ES => format!("{h} h"),
        _ => format!("{h}:00"),
    }
}

/// Tailles de fichiers : Ko/Mo/Go en français, KB/MB/GB ailleurs.
pub fn size_units() -> [&'static str; 3] {
    if lang() == FR {
        ["Ko", "Mo", "Go"]
    } else {
        ["KB", "MB", "GB"]
    }
}

/// Police chinoise de Windows (chargée seulement quand l'interface est en chinois).
/// Lue une seule fois, puis partagée (pas de copie de plusieurs Mo).
pub fn cjk_font() -> Option<&'static [u8]> {
    static FONT: OnceLock<Option<Vec<u8>>> = OnceLock::new();
    FONT.get_or_init(|| {
        let dir = std::env::var("WINDIR").unwrap_or_else(|_| r"C:\Windows".to_string());
        for name in ["msyh.ttc", "msyh.ttf", "simhei.ttf", "simsun.ttc", "Deng.ttf"] {
            if let Ok(b) = std::fs::read(std::path::Path::new(&dir).join("Fonts").join(name)) {
                return Some(b);
            }
        }
        crate::common::log("Chinese font not found in Windows\\Fonts: Chinese text may not display");
        None
    })
    .as_deref()
}
