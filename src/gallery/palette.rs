//! Palette de commandes (Ctrl+K) : on tape quelques lettres pour aller quelque part ou lancer une action
//! (onglets, rubriques des paramètres, thèmes, dossiers, tri, captures…), puis Entrée.
use super::*;

/// Ce que fait une commande de la palette.
#[derive(Clone)]
pub(super) enum Cmd {
    Tab(Tab),
    Cat(SetCat),
    Theme(&'static str),
    AccentAuto,
    Folder(String),
    FavsOnly,
    SortBy(Sort),
    Refresh,
    SelectAll,
    OpenFolder,
    Fullscreen,
    Sidebar,
    Open(usize),
}

pub(super) struct CmdPalette {
    query: String,
    sel: usize,
    opened: Instant,
    focus: bool, // donner le focus au champ de saisie à l'ouverture
}

impl CmdPalette {
    pub(super) fn new() -> Self {
        Self { query: String::new(), sel: 0, opened: Instant::now(), focus: true }
    }
}

/// Texte en minuscules et sans accents, pour comparer « parametres » et « Paramètres ».
fn norm(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| match c {
            'à' | 'â' | 'ä' | 'á' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'î' | 'ï' | 'í' => 'i',
            'ô' | 'ö' | 'ó' => 'o',
            'ù' | 'û' | 'ü' | 'ú' => 'u',
            'ç' => 'c',
            'ñ' => 'n',
            'ß' => 's',
            c => c,
        })
        .collect()
}

impl App {
    /// Toutes les commandes possibles : (icône, libellé, raccourci affiché, action).
    fn commands(&self, q: &str) -> Vec<(&'static str, String, String, Cmd)> {
        let mut v: Vec<(&'static str, String, String, Cmd)> = Vec::new();
        let go = tr("Aller à");
        v.push((icons::GALLERY, format!("{go} : {}", tr("Galerie")), String::new(), Cmd::Tab(Tab::Gallery)));
        v.push((icons::STATS, format!("{go} : {}", tr("Statistiques")), String::new(), Cmd::Tab(Tab::Stats)));
        v.push((icons::SETTINGS, format!("{go} : {}", tr("Paramètres")), String::new(), Cmd::Tab(Tab::Settings)));
        for (c, i, n) in [
            (SetCat::Shortcuts, icons::KEYBOARD, tr("Raccourcis")),
            (SetCat::Capture, icons::CAMERA, tr("Capture")),
            (SetCat::Files, icons::FOLDER, tr("Fichiers")),
            (SetCat::Look, icons::PALETTE, tr("Apparence")),
            (SetCat::Privacy, icons::SHIELD, tr("Confidentialité")),
            (SetCat::System, icons::SYSTEM, tr("Système")),
        ] {
            v.push((i, format!("{} › {n}", tr("Paramètres")), String::new(), Cmd::Cat(c)));
        }
        v.push((icons::REFRESH, tr("Actualiser").to_string(), String::new(), Cmd::Refresh));
        v.push((icons::SELECT_ALL, tr("Tout sélectionner").to_string(), "Ctrl+A".into(), Cmd::SelectAll));
        v.push((icons::FOLDER_OPEN, tr("Ouvrir le dossier des captures dans l'Explorateur").to_string(), String::new(), Cmd::OpenFolder));
        v.push((icons::STAR, tr("Afficher seulement les favoris").to_string(), String::new(), Cmd::FavsOnly));
        for (s, n) in [(Sort::Newest, tr("Plus récentes")), (Sort::Oldest, tr("Plus anciennes")), (Sort::Name, tr("Nom")), (Sort::Size, tr("Taille"))] {
            v.push((icons::SORT, format!("{} : {n}", tr("Trier")), String::new(), Cmd::SortBy(s)));
        }
        v.push((icons::FULLSCREEN, tr("Plein écran").to_string(), "F11".into(), Cmd::Fullscreen));
        v.push((icons::SIDEBAR, tr("Afficher / réduire le menu").to_string(), "Ctrl+B".into(), Cmd::Sidebar));
        for (key, name) in THEMES.iter() {
            v.push((icons::PALETTE, format!("{} : {}", tr("Thème"), tr(name)), String::new(), Cmd::Theme(key)));
        }
        v.push((icons::SPARKLES, format!("{} : {}", tr("Couleur d'accent"), tr("Auto|accent")), String::new(), Cmd::AccentAuto));
        v.push((icons::FOLDER, format!("{} : {}", tr("Dossier"), tr("Tous les dossiers")), String::new(), Cmd::Folder(String::new())));
        let mut folders: Vec<&String> = self.items.iter().map(|i| &i.folder).filter(|f| !f.is_empty()).collect();
        folders.sort();
        folders.dedup();
        for f in folders {
            v.push((icons::FOLDER, format!("{} : {f}", tr("Dossier")), String::new(), Cmd::Folder(f.clone())));
        }
        // filtrage : tous les mots tapés doivent apparaître, les libellés qui commencent par le texte d'abord
        let qn = norm(q);
        let words: Vec<&str> = qn.split_whitespace().collect();
        let mut out: Vec<(usize, (&'static str, String, String, Cmd))> = v
            .into_iter()
            .filter_map(|c| {
                let l = norm(&c.1);
                if words.iter().all(|w| l.contains(w)) {
                    let rank = if !qn.is_empty() && l.split(|ch: char| !ch.is_alphanumeric()).any(|part| part.starts_with(words[0])) { 0 } else { 1 };
                    Some((rank, c))
                } else {
                    None
                }
            })
            .collect();
        out.sort_by_key(|x| x.0);
        let mut out: Vec<_> = out.into_iter().map(|x| x.1).collect();
        // captures dont le nom correspond (6 au plus)
        if !qn.is_empty() {
            let mut n = 0;
            for (i, it) in self.items.iter().enumerate() {
                let name = file_name(&it.path);
                if words.iter().all(|w| norm(&name).contains(w)) {
                    out.push((icons::IMAGE, format!("{} : {name}", tr("Ouvrir")), String::new(), Cmd::Open(i)));
                    n += 1;
                    if n == 6 {
                        break;
                    }
                }
            }
        }
        out
    }

    /// Écrit un seul réglage dans la configuration enregistrée, sans enregistrer les autres modifications en cours.
    fn save_one(&mut self, f: impl Fn(&mut Config)) {
        f(&mut self.cfg);
        let mut disk = Config::load();
        f(&mut disk);
        disk.save();
        if let Ok(mut base) = toml::from_str::<Config>(&self.saved) {
            f(&mut base);
            self.saved = toml::to_string(&base).unwrap_or_default();
        }
    }

    fn run_cmd(&mut self, ctx: &egui::Context, c: Cmd) {
        match c {
            Cmd::Tab(t) => {
                self.view = None;
                self.tab = t;
            }
            Cmd::Cat(c) => {
                self.view = None;
                self.tab = Tab::Settings;
                self.set_cat = c;
            }
            Cmd::Theme(k) => self.save_one(|c| c.theme = k.to_string()),
            Cmd::AccentAuto => self.save_one(|c| c.accent_auto = true),
            Cmd::Folder(f) => {
                self.tab = Tab::Gallery;
                self.view = None;
                self.folder = f;
            }
            Cmd::FavsOnly => {
                self.tab = Tab::Gallery;
                self.view = None;
                self.favs_only = !self.favs_only;
            }
            Cmd::SortBy(s) => {
                self.tab = Tab::Gallery;
                self.sort = s;
            }
            Cmd::Refresh => self.items = list_items(&self.cfg.save_dir),
            Cmd::SelectAll => {
                self.tab = Tab::Gallery;
                self.view = None;
                self.sel = self.visible().iter().map(|&i| self.items[i].path.clone()).collect();
            }
            Cmd::OpenFolder => {
                let _ = std::fs::create_dir_all(&self.cfg.save_dir);
                let _ = Command::new("explorer").arg(&self.cfg.save_dir).spawn();
            }
            Cmd::Fullscreen => self.toggle_fullscreen(ctx),
            Cmd::Sidebar => self.sidebar_open = !self.sidebar_open,
            Cmd::Open(i) => {
                self.tab = Tab::Gallery;
                self.search.clear();
                self.folder.clear();
                self.favs_only = false;
                self.view = Some(i);
                self.view_dir = 0.0;
            }
        }
    }

    /// Clavier de la palette, à traiter en début d'image (avant les autres raccourcis) :
    /// Ctrl+K ouvre / ferme, ↑ ↓ choisissent, Entrée lance, Échap ferme.
    pub(super) fn palette_keys(&mut self, ctx: &egui::Context) -> Option<Cmd> {
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, Key::K)) {
            self.cmd = if self.cmd.is_some() { None } else { Some(CmdPalette::new()) };
        }
        let q = self.cmd.as_ref()?.query.clone();
        let n = self.commands(&q).len();
        let (up, down, enter, esc) = ctx.input_mut(|i| {
            (
                i.consume_key(egui::Modifiers::NONE, Key::ArrowUp),
                i.consume_key(egui::Modifiers::NONE, Key::ArrowDown),
                i.consume_key(egui::Modifiers::NONE, Key::Enter),
                i.consume_key(egui::Modifiers::NONE, Key::Escape),
            )
        });
        let p = self.cmd.as_mut()?;
        if esc {
            self.cmd = None;
            return None;
        }
        if n > 0 {
            if down {
                p.sel = (p.sel + 1) % n;
            }
            if up {
                p.sel = (p.sel + n - 1) % n;
            }
        }
        if enter && n > 0 {
            let sel = p.sel.min(n - 1);
            let c = self.commands(&q).swap_remove(sel).3;
            self.cmd = None;
            return Some(c);
        }
        None
    }

    /// Dessin de la palette (par-dessus tout).
    pub(super) fn palette_ui(&mut self, ctx: &egui::Context) {
        let opened = match &self.cmd {
            Some(p) => p.opened,
            None => return,
        };
        let k = ease_out((opened.elapsed().as_secs_f32() / 0.22).min(1.0));
        if k < 1.0 {
            ctx.request_repaint();
        }
        let screen = ctx.screen_rect();
        let mut chosen: Option<Cmd> = None;
        let mut close = false;
        egui::Area::new(egui::Id::new("cmd_palette")).order(egui::Order::Foreground).fixed_pos(screen.min).show(ctx, |ui| {
            // fond assombri : un clic dessus ferme la palette
            let (_, dim) = ui.allocate_exact_size(screen.size(), Sense::click());
            ui.painter().rect_filled(screen, 0.0, Color32::from_black_alpha((110.0 * k) as u8));
            if dim.clicked() {
                close = true;
            }
            let w = 600.0_f32.min(screen.width() - 32.0);
            let top = screen.min.y + screen.height() * 0.14 + 16.0 * (1.0 - k);
            let card = Rect::from_min_size(pos2(screen.center().x - w / 2.0, top), vec2(w, 10.0));
            let mut cu = ui.new_child(egui::UiBuilder::new().max_rect(Rect::from_min_size(card.min, vec2(w, screen.height() * 0.7))));
            cu.multiply_opacity(k);
            egui::Frame::popup(cu.style()).inner_margin(egui::Margin::same(10.0)).rounding(14.0_f32).show(&mut cu, |ui| {
                ui.set_width(w - 20.0);
                let p = self.cmd.as_mut().unwrap();
                let te = egui::TextEdit::singleline(&mut p.query)
                    .hint_text(ic(icons::COMMAND, tr("Tape une commande, un thème, un dossier ou une capture…")))
                    .font(FontId::proportional(17.0))
                    .frame(false)
                    .desired_width(f32::INFINITY)
                    .margin(vec2(8.0, 8.0));
                let r = ui.add(te);
                if p.focus {
                    r.request_focus();
                    p.focus = false;
                }
                if r.changed() {
                    p.sel = 0;
                }
                ui.separator();
                let q = p.query.clone();
                let sel = p.sel;
                let list = self.commands(&q);
                let sel = sel.min(list.len().saturating_sub(1));
                if list.is_empty() {
                    ui.add_space(6.0);
                    ui.weak(format!("  {}", tr("Aucune commande ne correspond.")));
                    ui.add_space(6.0);
                }
                let acc = self.shown_acc;
                egui::ScrollArea::vertical().max_height(screen.height() * 0.5).show(ui, |ui| {
                    for (n, (icon, label, hint, c)) in list.into_iter().enumerate() {
                        let (row, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 36.0), Sense::click());
                        let on = n == sel;
                        if on {
                            ui.scroll_to_rect(row, None);
                        }
                        let v = ui.visuals();
                        let pnt = ui.painter();
                        if on {
                            pnt.rect_filled(row, 8.0, Color32::from_rgba_unmultiplied(acc.r(), acc.g(), acc.b(), 50));
                        } else if resp.hovered() {
                            pnt.rect_filled(row, 8.0, v.widgets.hovered.weak_bg_fill);
                        }
                        let txt = v.text_color();
                        pnt.text(row.left_center() + vec2(14.0, 0.0), Align2::LEFT_CENTER, icon, FontId::proportional(17.0), if on { acc } else { txt });
                        pnt.text(row.left_center() + vec2(44.0, 0.0), Align2::LEFT_CENTER, &label, FontId::proportional(15.0), txt);
                        if on {
                            pnt.text(row.right_center() - vec2(12.0, 0.0), Align2::RIGHT_CENTER, icons::CORNER, FontId::proportional(15.0), v.weak_text_color());
                        } else if !hint.is_empty() {
                            pnt.text(row.right_center() - vec2(12.0, 0.0), Align2::RIGHT_CENTER, &hint, FontId::proportional(12.5), v.weak_text_color());
                        }
                        if resp.hovered() && ui.input(|i| i.pointer.is_moving()) {
                            if let Some(p) = self.cmd.as_mut() {
                                p.sel = n;
                            }
                        }
                        if resp.clicked() {
                            chosen = Some(c);
                        }
                    }
                });
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.add_space(6.0);
                    ui.weak(format!("{}  {}    {}  {}    {}  {}", icons::ARROW_UP_DOWN, tr("choisir"), icons::CORNER, tr("valider"), tr("Échap"), tr("fermer")));
                });
            });
        });
        if close {
            self.cmd = None;
        }
        if let Some(c) = chosen {
            self.cmd = None;
            self.run_cmd(ctx, c);
        }
    }

    /// Lance la commande choisie au clavier (appelé après `palette_keys`).
    pub(super) fn run_pending_cmd(&mut self, ctx: &egui::Context, c: Option<Cmd>) {
        if let Some(c) = c {
            self.run_cmd(ctx, c);
        }
    }
}
