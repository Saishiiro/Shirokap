#!/usr/bin/env python3
"""Reconstruit assets/fonts/Lucide-subset.ttf : la police d'icônes de Shirokap.

Elle ne contient que les icônes Lucide listées dans src/icons.rs. Chaque ligne y donne le
caractère utilisé par l'appli et, en commentaire, le nom de l'icône Lucide :

    pub const STAR: &str = "\\u{f706}"; // star

Pour ajouter une icône : ajouter une ligne à la fin de src/icons.rs (caractère suivant, nom pris
sur https://lucide.dev/icons), puis relancer ce script.

Il faut Python 3 avec fontTools (pip install fonttools) et le paquet npm lucide-static, qui
contient la police complète (font/lucide.ttf) et la liste de ses caractères (font/info.json) :

    npm pack lucide-static@1.81.0 && tar xzf lucide-static-1.81.0.tgz
    python tools/build-icons.py package

(1.81.0 est la version utilisée pour les icônes actuelles ; une version plus récente convient
aussi tant qu'elle a toutes les icônes de la liste.)
"""
import json
import re
import sys
from pathlib import Path

from fontTools import subset
from fontTools.ttLib import TTFont

root = Path(__file__).resolve().parent.parent
if len(sys.argv) != 2:
    sys.exit("usage : python tools/build-icons.py <dossier du paquet lucide-static>")
lucide = Path(sys.argv[1])

src = (root / "src" / "icons.rs").read_text(encoding="utf-8")
pairs = [(int(cp, 16), name) for cp, name in re.findall(r'"\\u\{([0-9a-f]+)\}"; // ([a-z0-9-]+)', src)]
info = json.loads((lucide / "font" / "info.json").read_text(encoding="utf-8"))
missing = [name for _, name in pairs if name not in info]
if missing:
    sys.exit("icônes absentes de ce paquet lucide-static : " + ", ".join(missing))

# mêmes contours, boîtes et date que lucide.ttf (pas de recalcul)
font = TTFont(lucide / "font" / "lucide.ttf", recalcBBoxes=False, recalcTimestamp=False)  # même résultat à chaque fois
cmap = font.getBestCmap()
for table in ["GSUB", "GPOS", "GDEF"]:
    if table in font:
        del font[table]  # ligatures de lucide.ttf, inutiles ici
# chaque caractère de src/icons.rs pointe vers le dessin de l'icône Lucide du même nom
mapping = {cp: cmap[int(info[name]["encodedCode"].lstrip("\\"), 16)] for cp, name in pairs}
for t in font["cmap"].tables:
    if t.isUnicode():
        t.cmap = dict(mapping)

opts = subset.Options()
opts.glyph_names = False
opts.name_IDs = ["*"]
opts.notdef_outline = True
opts.layout_features = []
sub = subset.Subsetter(opts)
sub.populate(unicodes=list(mapping))
sub.subset(font)
out = root / "assets" / "fonts" / "Lucide-subset.ttf"
font.save(out)
print(f"{len(pairs)} icônes -> {out.relative_to(root)}")
