<div align="center">

<img src="assets/logo.png" width="96" alt="Shirokap">

# Shirokap

**A lightweight screenshot tool for Windows, living in the system tray.**
Global hotkeys · gallery · annotation editor · 12 themes · EN / FR / ES / DE / 中文

[![Release](https://img.shields.io/github/v/release/Saishiiro/Shirokap?style=flat-square)](../../releases/latest)
[![GPL-3.0 License](https://img.shields.io/badge/license-GPL--3.0-blue?style=flat-square)](LICENSE)
![Windows 10/11](https://img.shields.io/badge/Windows-10%20%7C%2011-0078d4?style=flat-square)
![Rust](https://img.shields.io/badge/Rust-%23dea584?style=flat-square&logo=rust&logoColor=black)

[English](#english) · [Français](#français)

</div>

## English

### Download

Go to the [**Releases**](../../releases/latest) page and pick one:

- `Shirokap-Setup-x.y.z.exe`: installer (no administrator rights needed, optional launch with Windows);
- `Shirokap-x.y.z-portable.exe` (or `.zip`): portable version, a single file, no installation.

Windows may show a SmartScreen warning because the executable is not signed: click *More info*, then *Run anyway*.

Some browsers or antivirus programs may also warn when downloading an unsigned file from a new project: it is a false positive (Windows Defender finds nothing). The installer, which is a compressed package, is the file most often flagged by a few antivirus engines; the portable version is not. If you are worried, use the portable version. The full source code is in this repository and you can build it yourself with `build.bat`.

> Light by design: the resident process only handles the tray icon and the hotkeys. The gallery opens in a separate process that frees its memory when closed.

A lightweight screenshot tool for Windows, written in Rust. It runs quietly in the notification area (the "tray"), captures with a keyboard shortcut, and offers a gallery, an annotation editor and detailed settings.

### Features

**Capture**
- Full screen (main screen, screen under the mouse, or all screens), active window, or a region selected with the mouse (on any screen).
- True 4K resolution, with optional resizing (4K, 1440p, 1080p, 720p).
- Region to clipboard (no file), "redo the last region", color picker (copies the color under the cursor).
- Delay before capture, timelapse, blocklist of sensitive windows, automatic cleanup of old screenshots.

**Gallery**
- Grid of rounded thumbnails grouped by day, with search, sorting, favorites, multi-selection (with select all), quick access to the folder and drag and drop to import images.
- Viewer (zoom, slideshow, fullscreen), slider comparison, statistics.
- Collapsible side menu (Ctrl+B), 12 themes and a customizable accent color.

**Annotation editor**
- Pen, shapes (rectangle, ellipse, line, arrows…), highlighter, blur, text, numbered steps, eraser.
- Preview faithful to the final result, unlimited undo and redo, save as a copy or replace the original.

**Languages**
- English and French, plus Spanish, German and Chinese (unreviewed translations).
- Set in Settings > System > Language; "Automatic" follows the Windows language (English by default).
- Chinese uses the Windows font (Microsoft YaHei), loaded only when needed.

**Other**
- Screenshot pinned on top of the screen, copy files to the clipboard, export the selection.
- Launch with Windows, export/import of the configuration.

### Default hotkeys (editable in Settings > Shortcuts)

| Action | Hotkey |
|---|---|
| Capture the screen | `F12` |
| Capture a region | `Shift+F12` |
| Capture the active window | `Alt+F12` |
| Redo the last region | `Ctrl+F12` |
| Open the gallery | `Ctrl+Shift+F12` |
| Region to clipboard | `Shift+F11` |
| Color picker | `Alt+F11` |

Tray icon: left click = gallery, middle click = capture, right click = menu (modern style matching the app, or the native Windows menu: choose it in Settings > Appearance).

### Build

Requirements: Windows 10/11, [Rust](https://rustup.rs) and the Visual Studio "Build Tools" (C++ component).

```
build.bat
```

The script builds in release mode and places `shirokap.exe` at the project root. The executable is standalone: it needs no extra DLL.

Without installing Rust, the GitHub Actions workflow (`.github/workflows/build.yml`) builds the executable and the installer on GitHub's servers (*Actions* tab, then *Artifacts*).

### Customize the logo and the sound

Replace the files in `assets/` (same names), then run `build.bat` again: `logo.png`, `logo.ico`, `beep.wav`. See `assets/README.txt`.

### Installer (optional)

1. Install [Inno Setup 6](https://jrsoftware.org/isinfo.php).
2. Run `build.bat`, then `installer\build-installer.bat`.
3. The installer `Shirokap-Setup-<version>.exe` is created in `dist\`.

### Where is the data?

| Content | Location |
|---|---|
| Screenshots | `Pictures\Shirokap` (editable) |
| Configuration | `%APPDATA%\Shirokap\config\config.toml` |
| Favorites, interface state | `%APPDATA%\Shirokap\config\` |
| Thumbnails (can be deleted) | `%LOCALAPPDATA%\Shirokap\data\thumbs` |
| Log | `%LOCALAPPDATA%\Shirokap\data\logs\shirokap.log` |

### Troubleshooting

Open **Settings > System > Troubleshooting > Open the log**. It contains errors and crashes (capture failed, hotkey already in use, unreadable file…). Attach it when you report a bug.

### Code layout

```
src/
  main.rs        entry point: daemon, gallery, region or pin depending on the arguments
  daemon.rs      tray icon, global hotkeys, captures
  common.rs      configuration, image saving, log
  region.rs      region capture (fullscreen window on the chosen screen)
  pin.rs         screenshot pinned on top
  trayui.rs      modern tray menu (window drawn by the app)
  editor.rs      annotation editor
  picker.rs      color picker
  i18n.rs        languages: choice, Windows detection, dates, plurals
  i18n_table.rs  translation table (one row per text)
  gallery/
    mod.rs       application state and main loop
    grid.rs      thumbnail grid
    views.rs     viewer, comparison, statistics
    settings.rs  settings page
    ui_kit.rs    themes, palettes and interface components
    hotkeys.rs   hotkey recorder
```

### Changelog

See `CHANGELOG.md`.

### Licenses

- Shirokap is free software released under the **GNU General Public License v3.0** (`LICENSE`), © 2026 Saishiiro. You can use, share and modify it freely; any redistributed version, modified or not, must stay under the same license and come with its source code.
- **Inter** font © Rasmus Andersson, SIL Open Font License 1.1 (`assets/fonts/LICENSE-Inter.txt`).
- The Rust libraries used (egui, xcap, tray-icon, global-hotkey, image, etc.) have their own licenses, mostly MIT and Apache-2.0.

---

## Français

### Télécharger

Va sur la page [**Releases**](../../releases/latest) et choisis :

- `Shirokap-Setup-x.y.z.exe` : installateur (sans droits administrateur, lancement automatique avec Windows en option) ;
- `Shirokap-x.y.z-portable.exe` (ou `.zip`) : version portable, un seul fichier, aucune installation.

Windows peut afficher un avertissement SmartScreen car l'exécutable n'est pas signé : clique sur *Informations complémentaires* puis *Exécuter quand même*.

Certains navigateurs ou antivirus peuvent aussi signaler le téléchargement d'un fichier non signé issu d'un nouveau projet : c'est un faux positif (Windows Defender ne trouve rien). L'installateur, qui est un paquet compressé, est le fichier le plus souvent signalé par quelques antivirus ; la version portable ne l'est pas. Si tu as un doute, utilise la version portable. Tout le code source est dans ce dépôt et tu peux le compiler toi-même avec `build.bat`.

> Légère par conception : le processus résident ne fait que l'icône du tray et les raccourcis ; la galerie s'ouvre dans un processus séparé qui libère sa mémoire à la fermeture.

Outil de capture d'écran léger pour Windows, écrit en Rust. Il tourne discrètement dans la zone de notification
(le « tray »), capture au raccourci clavier, et propose une galerie, un éditeur d'annotations et des réglages détaillés.

### Fonctionnalités

**Capture**
- Plein écran (écran principal, écran sous la souris ou tous les écrans), fenêtre active, zone à la souris (sur n'importe quel écran).
- Vraie résolution 4K, avec option de redimensionnement (4K, 1440p, 1080p, 720p).
- Zone vers le presse-papiers (sans fichier), « refaire la dernière zone », pipette (copie la couleur sous le curseur).
- Délai avant capture, timelapse, liste noire de fenêtres sensibles, rangement automatique des vieilles captures.

**Galerie**
- Grille de miniatures arrondies, regroupées par jour, avec recherche, tri, favoris, sélection multiple (avec « Tout sélectionner »), accès direct au dossier et glisser-déposer pour importer des images.
- Visionneuse (zoom, diaporama, plein écran), comparateur à curseur, statistiques.
- Menu latéral réductible (Ctrl+B), 12 thèmes et couleur d'accent personnalisable.

**Éditeur d'annotations**
- Stylo, formes (rectangle, ellipse, ligne, flèches…), surligneur, flou, texte, étapes numérotées, gomme.
- Aperçu fidèle au résultat final, annulation et rétablissement illimités, enregistrement en copie ou remplacement de l'original.

**Langues**
- Français et anglais, plus espagnol, allemand et chinois (traductions non vérifiées).
- Réglage dans Paramètres > Système > Langue ; « Automatique » suit la langue de Windows (anglais par défaut).
- Le chinois utilise la police de Windows (Microsoft YaHei), chargée seulement quand elle est nécessaire.

**Autres**
- Capture épinglée au premier plan de l'écran, copie des fichiers dans le presse-papiers, export de la sélection.
- Lancement automatique avec Windows, export/import de la configuration.

### Raccourcis par défaut (modifiables dans Paramètres > Raccourcis)

| Action | Raccourci |
|---|---|
| Capturer l'écran | `F12` |
| Capturer une zone | `Shift+F12` |
| Capturer la fenêtre active | `Alt+F12` |
| Refaire la dernière zone | `Ctrl+F12` |
| Ouvrir la galerie | `Ctrl+Shift+F12` |
| Zone vers le presse-papiers | `Shift+F11` |
| Pipette (couleur) | `Alt+F11` |

Icône du tray : clic gauche = galerie, clic molette = capture, clic droit = menu (style moderne aux couleurs de l'appli, ou menu natif de Windows : à choisir dans Paramètres > Apparence).

### Compiler

Prérequis : Windows 10/11, [Rust](https://rustup.rs) et les « Build Tools » de Visual Studio (composant C++).

```
build.bat
```

Le script compile en mode release et dépose `shirokap.exe` à la racine du projet. L'exécutable est autonome :
il n'a besoin d'aucune DLL supplémentaire.

Sans installer Rust, le workflow GitHub Actions (`.github/workflows/build.yml`) compile l'exécutable et l'installateur
sur les serveurs de GitHub (onglet *Actions*, puis *Artifacts*).

### Personnaliser le logo et le son

Remplace les fichiers de `assets/` (mêmes noms), puis relance `build.bat` : `logo.png`, `logo.ico`, `beep.wav`.
Voir `assets/README.txt`.

### Installateur (optionnel)

1. Installe [Inno Setup 6](https://jrsoftware.org/isinfo.php).
2. Lance `build.bat`, puis `installer\build-installer.bat`.
3. L'installateur `Shirokap-Setup-<version>.exe` est créé dans `dist\`.

### Où sont les données ?

| Contenu | Emplacement |
|---|---|
| Captures | `Images\Shirokap` (modifiable) |
| Configuration | `%APPDATA%\Shirokap\config\config.toml` |
| Favoris, état de l'interface | `%APPDATA%\Shirokap\config\` |
| Miniatures (supprimables) | `%LOCALAPPDATA%\Shirokap\data\thumbs` |
| Journal | `%LOCALAPPDATA%\Shirokap\data\logs\shirokap.log` |

### En cas de problème

Ouvre **Paramètres > Système > Dépannage > Ouvrir le journal**. Il contient les erreurs et les plantages
(capture impossible, raccourci déjà utilisé, fichier illisible…). Joins-le quand tu signales un bug.

### Organisation du code

```
src/
  main.rs        point d'entrée : démon, galerie, zone ou épingle selon les arguments
  daemon.rs      icône du tray, raccourcis globaux, captures
  common.rs      configuration, enregistrement des images, journal
  region.rs      capture de zone (fenêtre plein écran sur l'écran choisi)
  pin.rs         capture épinglée au premier plan
  trayui.rs      menu moderne de l'icône du tray (fenêtre dessinée par l'appli)
  editor.rs      éditeur d'annotations
  picker.rs      sélecteur de couleur
  i18n.rs        langues : choix, détection de Windows, dates, pluriels
  i18n_table.rs  tableau des traductions (une ligne par texte)
  gallery/
    mod.rs       état de l'application et boucle principale
    grid.rs      grille de miniatures
    views.rs     visionneuse, comparateur, statistiques
    settings.rs  page des paramètres
    ui_kit.rs    thèmes, palettes et composants d'interface
    hotkeys.rs   enregistreur de raccourci
```

### Historique

Voir `CHANGELOG.md`.

### Licences

- Shirokap est un logiciel libre publié sous licence **GNU General Public License v3.0** (`LICENSE`), © 2026 Saishiiro. Tu peux l'utiliser, le partager et le modifier librement ; toute version redistribuée, modifiée ou non, doit rester sous la même licence et être fournie avec son code source.
- Police **Inter** © Rasmus Andersson, licence SIL Open Font License 1.1 (`assets/fonts/LICENSE-Inter.txt`).
- Les bibliothèques Rust utilisées (egui, xcap, tray-icon, global-hotkey, image, etc.) ont leurs propres licences,
  principalement MIT et Apache-2.0.
