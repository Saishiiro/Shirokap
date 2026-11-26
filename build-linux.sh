#!/usr/bin/env bash
# Compile Shirokap pour Windows depuis Linux (compilation croisée avec MinGW).
# Produit dans dist/ : la version portable (.exe et .zip), l'installateur (si Wine
# et Inno Setup sont installés) et SHA256SUMS.txt.
#
#   ./build-linux.sh                 tout construire
#   ./build-linux.sh --no-installer  seulement la version portable
#
# Prérequis (Arch) : sudo pacman -S rustup mingw-w64-gcc zip
#                    rustup default stable && rustup target add x86_64-pc-windows-gnu
# Installateur     : sudo pacman -S wine, puis installer Inno Setup 6 dans Wine (voir README).
set -euo pipefail
cd "$(dirname "$0")"

TARGET=x86_64-pc-windows-gnu
INSTALLER=1
[[ "${1:-}" == "--no-installer" ]] && INSTALLER=0

fail() { echo "ERREUR : $*" >&2; exit 1; }
need() { command -v "$1" >/dev/null 2>&1 || fail "$1 est introuvable. $2"; }

VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)
[[ -n "$VERSION" ]] || fail "version introuvable dans Cargo.toml"
echo "== Shirokap $VERSION =="

# 1. Vérifications
need cargo "Installe Rust : https://rustup.rs"
need x86_64-w64-mingw32-gcc "Installe MinGW : sudo pacman -S mingw-w64-gcc"
need x86_64-w64-mingw32-windres "Installe MinGW : sudo pacman -S mingw-w64-gcc"
if command -v rustup >/dev/null 2>&1 && ! rustup target list --installed | grep -qx "$TARGET"; then
    fail "la cible $TARGET n'est pas installée. Lance : rustup target add $TARGET"
fi

# 2. Compilation de l'exe Windows
echo "== Compilation (cela peut prendre quelques minutes la première fois) =="
cargo build --release --target "$TARGET"
EXE="target/$TARGET/release/shirokap.exe"
[[ -f "$EXE" ]] || fail "$EXE n'a pas été créé"

# 3. Version portable
mkdir -p dist
PORTABLE="Shirokap-$VERSION-portable"
cp "$EXE" "dist/$PORTABLE.exe"
cp "$EXE" shirokap.exe # utilisé par l'installateur (ignoré par git)
rm -f "dist/$PORTABLE.zip"
if command -v zip >/dev/null 2>&1; then
    (cd dist && zip -q "$PORTABLE.zip" "$PORTABLE.exe")
elif command -v bsdtar >/dev/null 2>&1; then
    (cd dist && bsdtar -a -cf "$PORTABLE.zip" "$PORTABLE.exe")
else
    fail "zip est introuvable. Installe-le : sudo pacman -S zip"
fi
echo "OK : dist/$PORTABLE.exe et dist/$PORTABLE.zip"

# 4. Installateur (Inno Setup dans Wine)
if [[ $INSTALLER == 1 ]]; then
    PREFIX="${WINEPREFIX:-$HOME/.wine}"
    ISCC=""
    for d in "Program Files (x86)" "Program Files"; do
        [[ -f "$PREFIX/drive_c/$d/Inno Setup 6/ISCC.exe" ]] && ISCC="$PREFIX/drive_c/$d/Inno Setup 6/ISCC.exe"
    done
    [[ -z "$ISCC" && -f "$PREFIX/drive_c/users/${USER:-$(id -un)}/AppData/Local/Programs/Inno Setup 6/ISCC.exe" ]] \
        && ISCC="$PREFIX/drive_c/users/${USER:-$(id -un)}/AppData/Local/Programs/Inno Setup 6/ISCC.exe"
    if ! command -v wine >/dev/null 2>&1 || [[ -z "$ISCC" ]]; then
        echo "Installateur ignoré : Wine ou Inno Setup 6 (dans Wine) est introuvable."
        echo "  Voir la section « Compiler sous Linux » du README, ou relance avec --no-installer."
    else
        echo "== Installateur (Inno Setup via Wine) =="
        rm -f "dist/Shirokap-Setup-$VERSION.exe"
        WINEDEBUG=-all wine "$ISCC" /Q "/DMyAppVersion=$VERSION" "$(winepath -w installer/Shirokap.iss 2>/dev/null)"
        [[ -f "dist/Shirokap-Setup-$VERSION.exe" ]] || fail "l'installateur n'a pas été créé"
        echo "OK : dist/Shirokap-Setup-$VERSION.exe"
    fi
fi

# 5. Empreintes
(cd dist && sha256sum Shirokap-"$VERSION"-* Shirokap-Setup-"$VERSION".exe 2>/dev/null > SHA256SUMS.txt || true)
echo "== Empreintes SHA-256 (dist/SHA256SUMS.txt) =="
cat dist/SHA256SUMS.txt
