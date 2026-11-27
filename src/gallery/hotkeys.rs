// Enregistreur de raccourci : lecture directe du clavier Windows.
use super::*;

// ---------- enregistreur de raccourci (lecture directe du clavier Windows) ----------
pub(super) enum Poll {
    Wait,
    Cancel,
    Clear,
    Combo(String),
}

pub(super) fn vk_down(vk: i32) -> bool {
    unsafe { (GetAsyncKeyState(vk) as u16 & 0x8000) != 0 }
}

/// Nom de touche compris par global-hotkey, et si elle peut servir seule (sans Ctrl/Alt/Maj).
pub(super) fn vk_name(vk: i32) -> Option<(String, bool)> {
    let n = |s: &str, free: bool| Some((s.to_string(), free));
    match vk {
        0x70..=0x87 => Some((format!("F{}", vk - 0x6F), true)),
        0x2C => n("PrintScreen", true),
        0x13 => n("Pause", true),
        0x91 => n("ScrollLock", true),
        0x41..=0x5A | 0x30..=0x39 => Some((((vk as u8) as char).to_string(), false)),
        0x60..=0x69 => Some((format!("Numpad{}", vk - 0x60), false)),
        0x20 => n("Space", false),
        0x09 => n("Tab", false),
        0x2D => n("Insert", false),
        0x2E => n("Delete", false),
        0x24 => n("Home", false),
        0x23 => n("End", false),
        0x21 => n("PageUp", false),
        0x22 => n("PageDown", false),
        0x25 => n("Left", false),
        0x26 => n("Up", false),
        0x27 => n("Right", false),
        0x28 => n("Down", false),
        0xC0 => n("Backquote", false),
        0xBD => n("Minus", false),
        0xBB => n("Equal", false),
        0xDB => n("BracketLeft", false),
        0xDD => n("BracketRight", false),
        0xBA => n("Semicolon", false),
        0xDE => n("Quote", false),
        0xBC => n("Comma", false),
        0xBE => n("Period", false),
        0xBF => n("Slash", false),
        0xDC => n("Backslash", false),
        _ => None,
    }
}

pub(super) fn poll_hotkey() -> Poll {
    if vk_down(0x1B) {
        return Poll::Cancel; // Échap
    }
    if vk_down(0x08) {
        return Poll::Clear; // Retour arrière
    }
    let mut mods: Vec<&str> = Vec::new();
    if vk_down(0x11) {
        mods.push("Ctrl");
    }
    if vk_down(0x10) {
        mods.push("Shift");
    }
    if vk_down(0x12) {
        mods.push("Alt");
    }
    if vk_down(0x5B) || vk_down(0x5C) {
        mods.push("Super");
    }
    for vk in 0x08..=0xDE {
        if let Some((name, _)) = vk_name(vk) {
            // n'importe quelle touche peut servir seule (l'avertissement s'affiche sous le champ)
            if vk_down(vk) {
                let mut parts: Vec<String> = mods.iter().map(|m| m.to_string()).collect();
                parts.push(name);
                return Poll::Combo(parts.join("+"));
            }
        }
    }
    Poll::Wait
}
