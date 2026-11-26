// Shirokap - lightweight screenshot tool for Windows
// Copyright (C) 2026 Saishiiro
//
// This program is free software: you can redistribute it and/or modify it under the terms of the
// GNU General Public License as published by the Free Software Foundation, version 3.
// This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY;
// without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
// See the LICENSE file for details.

#![windows_subsystem = "windows"] // pas de fenêtre de console

mod common;
mod daemon;
mod editor;
mod gallery;
mod i18n;
mod icons;
mod i18n_table;
mod picker;
mod pin;
mod region;
mod trayui;
mod zoom;

fn main() {
    common::dpi_aware(); // indispensable pour capturer en vraie résolution (4K) avec la mise à l'échelle Windows
    let a: Vec<String> = std::env::args().collect();
    let arg = |i: usize| a.get(i + 1).cloned().unwrap_or_default();
    let region_pos = a.iter().position(|x| x == "--region" || x == "--region-clip");
    let pin_pos = a.iter().position(|x| x == "--pin");
    let gui = a.iter().any(|x| x == "--gui");
    let menu_pos = a.iter().position(|x| x == "--menu");
    let mode = if menu_pos.is_some() {
        "menu"
    } else if region_pos.is_some() {
        "region"
    } else if pin_pos.is_some() {
        "pin"
    } else if gui {
        "gallery"
    } else {
        "daemon"
    };
    common::install_panic_hook(mode); // les plantages sont écrits dans le journal
    i18n::set_lang(&common::Config::load().language); // langue de l'interface
    if mode == "daemon" {
        common::log(&format!("daemon started (v{})", env!("CARGO_PKG_VERSION")));
    }
    if let Some(i) = menu_pos {
        let n = |k: usize| a.get(i + k).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0);
        trayui::run(n(1), n(2), n(3) == 1, n(4) == 1); // menu moderne du tray
    } else if let Some(i) = region_pos {
        region::run(arg(i), a[i] == "--region-clip"); // overlay de capture de zone
    } else if let Some(i) = pin_pos {
        pin::run(arg(i)); // capture épinglée à l'écran
    } else if gui {
        gallery::run(a.iter().any(|x| x == "--settings")); // galerie + paramètres
    } else {
        daemon::run(); // démon : tray + raccourcis + capture
    }
}
