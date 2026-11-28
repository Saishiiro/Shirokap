# Changelog · Historique des versions

[English](#english) · [Français](#français)

## English

### 1.3.0 — November 2026

A redesign of the whole interface: floating panels, named buttons, more information and fewer mistakes.

**New look**
- Floating rounded panels, pill navigation with the logo, a command bar ("Search for a command… Ctrl K") at the top of every page, and a "Save" pill that stays at the bottom of the window when settings change.
- New Shirokap theme, in the mauve of the logo with a white accent (18 themes).
- On first launch the theme follows Windows: Light or Dark.
- Narrow windows: the side menu folds by itself and toolbars wrap instead of overflowing.

**Gallery**
- New toolbar (search, sort, thumbnail size, folder and favorite chips) and a floating bar for the selection (compare, copy, export, favorite, Recycle Bin).
- Viewer: file name and details at the top, named buttons, round previous / next buttons, image in an inset panel, rename field validated with Enter.
- Slideshow: progress pill in the top bar (in full screen it shows when the mouse moves, like the zoom bar), and an adjustable time per image (Settings > Appearance > Viewer).
- Comparison: divider with a round handle, A and B labels, swap button.

**Statistics**
- Tiles for captures, space used, favorites, oldest and heaviest screenshot.
- Panels that fill the page: last 7 days, folders, an activity calendar with the current and best streaks, and capture hours.

**Safer**
- A confirmation is asked before anything goes to the Recycle Bin (selection, thumbnail button, right-click, viewer).
- Notifications stay a little longer and offer "Undo" after a move to the Recycle Bin (the files come back from the Windows Recycle Bin), a rename or a settings reset.

**Settings**
- Grouped rows, switches, pill selectors, steppers with − and +.
- Shortcuts can be typed or recorded, and a single key (F7, Pause…) works as well as a combination.
- About: copyright, GPL-3.0 license, link to the source code, and a window with the full license texts.

**Editor**
- Toolbar with named tools, a Shapes menu with a small drawing of each shape, tool settings on their own row.
- The zoom bar no longer hides the cursor or lets a stroke start under it.

**Menus**
- Command palette with groups, colored icons, theme swatches and screenshot thumbnails.
- Tray menu with a header, quick tiles (screen, region, gallery, color picker) and switches; "Settings…" now also works when the gallery is already open.
- Dropdowns stay inside the window and every menu row has an icon.
- Welcome screen redone: language and theme pickers, shortcuts, tray gestures, captures folder.

**Other**
- Clipboard copy after a capture is now off by default.
- Sizes use a decimal comma in French, Spanish and German.
- The installer also includes the GPL-3.0 and Lucide license files.

### 1.2.0 — November 2026

A big visual update of the gallery: more modern, animated, with icons.

**Animations**
- Thumbnails fade in as they load, appear in cascade, and lift with a soft shadow on hover.
- Opening a screenshot: it flies from its thumbnail to the viewer, and back when closing.
- Sliding side menu, notifications, selection bar; tab and settings changes with a fade.
- Statistics bars that grow, theme cards and accent dots with bounces and ripples.

**Viewer and editor**
- New zoom: wheel towards the cursor, drag to move, double-click (fit / actual size), + − 0 1, floating zoom bar.
- Zoom in the built-in editor too (move with middle-click or Space + drag).

**Look**
- 5 new themes (Glacier, Mocha, Neon, Sakura, Sky): 17 in total.
- Switching theme: a circle grows from the click and reveals the new theme.
- Each theme has its own accent color, with an "Auto" button to follow it.
- Lucide icons everywhere (menus, toolbars, editor, tray menu, notifications); the collapsed side menu shows the tab icons.
- Title bar matching the theme (Windows 11), soft shadows, themed scrollbars.

**Comfort**
- Command palette (Ctrl+K): tabs, settings sections, themes, folders, sorting, actions, and opening a screenshot by name or date.
- Empty states for an empty gallery, a search without results, no favorites, an empty folder and statistics.
- Shimmer while thumbnails load.

**Performance**
- Windows 11 power-saving mode (EcoQoS) for the tray process (except during a capture) and for the gallery in the background. The priority stays normal, so Task Manager does not show its "Efficiency mode" leaf.
- The list of displayed screenshots is cached instead of being recomputed every frame.

### 1.1.0 — November 2026

**Gallery**
- "Select all" button (becomes "Deselect all" when everything shown is selected).
- "Open folder" button: opens the screenshots folder (or the filtered subfolder) in File Explorer.

**Editor**
- "Redo" button next to "Undo", also with Ctrl+Y or Ctrl+Shift+Z.
- The "Undo" button was labeled "Cancel" in English and is now correct.

**Tray**
- "Quit" now closes the whole app: gallery, region capture and pinned screenshots too.

### 1.0.0 — November 2026

First stable release.

**Languages**
- Interface in English and French, plus Spanish, German and Chinese (unreviewed translations).
- "Automatic" mode following the Windows language (English by default), set in Settings > System.
- Dates, file sizes and plurals adapted to each language; Windows Chinese font loaded only when needed.
- Bilingual installer (French / English); log in English.

**Other**
- New, quieter capture sound.
- Slideshow order of your choice (oldest to newest by default).

### 0.12 — tray menu
- Modern tray icon menu: Inter font, rounded corners, theme colors, side submenus.
- Native Windows menu still available (Settings > Appearance).

### 0.11 — stability
- Error and crash log, available from Settings > System > Troubleshooting.
- Backup copy of an unreadable configuration.
- Gallery code split into several files, complete README.
- Inno Setup installer and automatic build on GitHub.

### 0.10 — settings and appearance
- Settings organized in categories, with an explanation for each option.
- Welcome screen on first launch.
- 12 themes, quick accent colors (including white and black), custom color picker.
- Reset per category or everything.

### 0.9 — editor
- Eraser, size circle under the cursor, blinking caret for text.
- "Shapes" menu: rectangle, ellipse (outlined or filled), line, arrow, double arrow.
- Confirmation before replacing the original.

### 0.6 to 0.8 — interface overhaul
- Collapsible sidebar (Ctrl+B), Inter font, restrained palette.
- Gallery: rounded thumbnails, hover buttons, file names, renaming, notifications.
- Viewer: grouped toolbar, fullscreen on the image only (F11).

### 0.5 — capture
- Region capture on any screen, text tool and numbered steps, hotkey recorder.
- Improved capture of fullscreen games (Windows Graphics Capture method).

### 0.4 — tools
- Annotation editor, pinned screenshot, region to clipboard, color picker, comparison, drag and drop, export.

### 0.3 — gallery and tray
- Timelapse, window blocklist, automatic cleanup, statistics, slideshow, new tray menu.

### 0.2 — first advanced features
- 4K capture, region and window capture, favorites, search, sorting, light and dark themes.

### 0.1 — first version
- Capture with a hotkey, notification-area icon, gallery and settings.

---

## Français

### 1.3.0 — novembre 2026

Une refonte de toute l'interface : des panneaux flottants, des boutons qui disent ce qu'ils font, plus d'informations et moins de fausses manœuvres.

**Nouveau look**
- Panneaux arrondis flottants, navigation en pilules avec le logo, barre « Rechercher une commande… Ctrl K » en haut de chaque page, et pilule « Enregistrer » qui reste en bas de la fenêtre quand des réglages changent.
- Nouveau thème Shirokap, au mauve du logo avec un accent blanc (18 thèmes).
- Au premier lancement, le thème suit Windows : Clair ou Sombre.
- Fenêtre étroite : le menu latéral se replie tout seul et les barres d'outils passent à la ligne au lieu de déborder.

**Galerie**
- Nouvelle barre d'outils (recherche, tri, taille des miniatures, pastilles de dossiers et favoris) et barre flottante pour la sélection (comparer, copier, exporter, favori, corbeille).
- Visionneuse : nom et détails de la capture en haut, boutons nommés, boutons ronds précédente / suivante, image dans un panneau en retrait, renommage validé avec Entrée.
- Diaporama : pilule de progression dans la barre du haut (en plein écran, elle apparaît quand la souris bouge, comme la barre de zoom) et durée par image réglable (Paramètres > Apparence > Visionneuse).
- Comparateur : séparation avec une poignée ronde, étiquettes A et B, bouton pour les inverser.

**Statistiques**
- Tuiles pour les captures, l'espace utilisé, les favoris, la plus ancienne et la plus lourde.
- Des panneaux qui remplissent la page : 7 derniers jours, dossiers, calendrier d'activité avec la série en cours et le record, et heures de capture.

**Plus sûr**
- Une confirmation est demandée avant toute mise à la corbeille (sélection, bouton d'une miniature, clic droit, visionneuse).
- Les notifications restent un peu plus longtemps et proposent « Annuler » après une mise à la corbeille (les fichiers reviennent de la corbeille de Windows), un renommage ou une réinitialisation des réglages.

**Paramètres**
- Lignes regroupées, interrupteurs, sélecteurs en pilule, champs numériques avec − et +.
- Les raccourcis peuvent être écrits ou enregistrés, et une touche seule (F7, Pause…) marche aussi bien qu'une combinaison.
- À propos : copyright, licence GPL-3.0, lien vers le code source, et une fenêtre avec le texte complet des licences.

**Éditeur**
- Barre d'outils avec des outils nommés, menu Formes avec un petit dessin de chaque forme, réglages de l'outil sur leur propre ligne.
- La barre de zoom ne cache plus la souris et on ne commence plus un trait sous elle.

**Menus**
- Palette de commandes avec des groupes, des icônes colorées, des aperçus de thèmes et des miniatures de captures.
- Menu du tray avec un en-tête, des tuiles rapides (écran, zone, galerie, pipette) et des interrupteurs ; « Paramètres… » marche aussi quand la galerie est déjà ouverte.
- Les menus déroulants restent dans la fenêtre et chaque ligne de menu a son icône.
- Écran d'accueil refait : choix de la langue et du thème, raccourcis, gestes de l'icône du tray, dossier des captures.

**Autres**
- La copie dans le presse-papiers après une capture est maintenant désactivée par défaut.
- Les tailles utilisent une virgule décimale en français, espagnol et allemand.
- L'installateur ajoute aussi les fichiers de licence GPL-3.0 et Lucide.

### 1.2.0 — novembre 2026

Une grosse mise à jour visuelle de la galerie : plus moderne, animée, avec des icônes.

**Animations**
- Les miniatures apparaissent en fondu, en cascade, et se soulèvent avec une ombre douce au survol.
- Ouverture d'une capture : elle vole de sa miniature à la visionneuse, et y retourne à la fermeture.
- Menu latéral, notifications et barre de sélection qui glissent ; changements d'onglet et de rubrique en fondu.
- Barres de statistiques qui poussent, cartes de thème et pastilles d'accent avec rebonds et ondes.

**Visionneuse et éditeur**
- Nouveau zoom : molette vers le curseur, glisser pour déplacer, double-clic (ajuster / taille réelle), + − 0 1, barre de zoom flottante.
- Zoom aussi dans l'éditeur intégré (déplacement au clic molette ou Espace + glisser).

**Apparence**
- 5 nouveaux thèmes (Glacier, Moka, Néon, Sakura, Ciel) : 17 au total.
- Changement de thème : un cercle part du clic et révèle le nouveau thème.
- Chaque thème a sa couleur d'accent, avec un bouton « Auto » pour la suivre.
- Icônes Lucide partout (menus, barres d'outils, éditeur, menu du tray, notifications) ; le menu latéral réduit affiche les icônes des onglets.
- Barre de titre assortie au thème (Windows 11), ombres douces, barres de défilement aux couleurs du thème.

**Confort**
- Palette de commandes (Ctrl+K) : onglets, rubriques des paramètres, thèmes, dossiers, tri, actions, et ouverture d'une capture par son nom ou sa date.
- Écrans vides soignés : galerie vide, recherche sans résultat, aucun favori, dossier vide, statistiques.
- Reflet animé pendant le chargement des miniatures.

**Performances**
- Mode économie d'énergie de Windows 11 (EcoQoS) pour le processus du tray (sauf pendant une capture) et pour la galerie en arrière-plan. La priorité reste normale : le gestionnaire des tâches n'affiche donc pas sa feuille « Mode efficacité ».
- La liste des captures affichées est gardée en mémoire au lieu d'être recalculée à chaque image.

### 1.1.0 — novembre 2026

**Galerie**
- Bouton « Tout sélectionner » (devient « Tout désélectionner » quand tout ce qui est affiché est sélectionné).
- Bouton « Ouvrir le dossier » : ouvre le dossier des captures (ou le sous-dossier filtré) dans l'Explorateur.

**Éditeur**
- Bouton « Rétablir » à côté d'« Annuler », aussi avec Ctrl+Y ou Ctrl+Maj+Z.
- Le bouton « Annuler » s'affichait « Cancel » en anglais au lieu de « Undo » : corrigé.

**Tray**
- « Quitter » ferme maintenant toute l'appli : la galerie, la capture de zone et les captures épinglées aussi.

### 1.0.0 — novembre 2026

Première version stable.

**Langues**
- Interface en français et en anglais, plus espagnol, allemand et chinois (traductions non vérifiées).
- Mode « Automatique » qui suit la langue de Windows (anglais par défaut), réglable dans Paramètres > Système.
- Dates, tailles de fichiers et pluriels adaptés à chaque langue ; police chinoise de Windows chargée seulement quand elle est nécessaire.
- Installateur bilingue (français / anglais) ; journal en anglais.

**Divers**
- Nouveau son de capture, plus discret.
- Ordre du diaporama au choix (du plus ancien au plus récent par défaut).

### 0.12 — menu du tray
- Menu moderne de l'icône du tray : police Inter, coins arrondis, couleurs du thème, sous-menus sur le côté.
- Menu natif de Windows toujours disponible (Paramètres > Apparence).

### 0.11 — stabilité
- Journal des erreurs et des plantages, accessible depuis Paramètres > Système > Dépannage.
- Copie de secours d'une configuration illisible.
- Code de la galerie découpé en plusieurs fichiers, README complet.
- Installateur Inno Setup et compilation automatique sur GitHub.

### 0.10 — paramètres et apparence
- Paramètres rangés en rubriques, avec une explication pour chaque réglage.
- Écran d'accueil au premier lancement.
- 12 thèmes, couleurs d'accent rapides (dont blanc et noir), sélecteur de couleur maison.
- Réinitialisation par rubrique ou complète.

### 0.9 — éditeur
- Gomme, cercle de taille sous le curseur, barre de saisie clignotante pour le texte.
- Menu « Formes » : rectangle, ellipse (vides ou pleins), ligne, flèche, flèche double.
- Confirmation avant de remplacer l'original.

### 0.6 à 0.8 — refonte de l'interface
- Barre latérale réductible (Ctrl+B), police Inter, palette sobre.
- Galerie : miniatures arrondies, boutons au survol, nom des fichiers, renommage, notifications.
- Visionneuse : barre d'outils regroupée, plein écran sur l'image seule (F11).

### 0.5 — capture
- Capture de zone sur n'importe quel écran, outil texte et étapes numérotées, enregistreur de raccourci.
- Capture des jeux en plein écran améliorée (méthode Windows Graphics Capture).

### 0.4 — outils
- Éditeur d'annotations, capture épinglée, zone vers le presse-papiers, pipette, comparateur, glisser-déposer, export.

### 0.3 — galerie et tray
- Timelapse, liste noire de fenêtres, rangement automatique, statistiques, diaporama, nouveau menu du tray.

### 0.2 — premières fonctions avancées
- Capture 4K, capture de zone et de fenêtre, favoris, recherche, tri, thèmes clair et sombre.

### 0.1 — première version
- Capture au raccourci, icône dans la zone de notification, galerie et paramètres.
