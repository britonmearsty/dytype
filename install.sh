#!/bin/sh
# dytype installer.
#
# Fetches a prebuilt dytype binary from the GitHub releases and installs it
# into ~/.local/bin (or a directory of your choice), verifying the download
# against the release's SHA256SUMS. Works on Linux, macOS, and Git Bash /
# MSYS2 / Cygwin on Windows.
#
# Usage:
#   sh install.sh              # latest release to ~/.local/bin
#   sh install.sh --dir PATH   # install into PATH/bin style directory
#   sh install.sh --version v0.1.0
#
# Environment overrides:
#   DYTYPE_REPO         GitHub "owner/repo" (default: britonmearsty/dytype)
#   DYTYPE_VERSION      release tag to install (default: latest)
#   DYTYPE_BASE_URL     download root (default: repo releases/download)
#   DYTYPE_INSTALL_DIR  install directory (default: $HOME/.local/bin)
set -eu

REPO="${DYTYPE_REPO:-britonmearsty/dytype}"
VERSION="${DYTYPE_VERSION:-}"
BASE_URL="${DYTYPE_BASE_URL:-https://github.com/${REPO}/releases/download}"
INSTALL_DIR="${DYTYPE_INSTALL_DIR:-${HOME}/.local/bin}"

usage() {
    sed -n '2,16p' "$0" | sed 's/^# \{0,1\}//'
}

die() {
    printf 'install.sh: error: %s\n' "$1" >&2
    exit 1
}

require_cmd() {
    command -v "$1" >/dev/null 2>&1
}

while [ "$#" -gt 0 ]; do
    case "$1" in
        -h | --help)
            usage
            exit 0
            ;;
        -d | --dir)
            [ "$#" -ge 2 ] || die "--dir requires a directory argument"
            INSTALL_DIR="$2"
            shift 2
            ;;
        --prefix)
            [ "$#" -ge 2 ] || die "--prefix requires a directory argument"
            INSTALL_DIR="$2/bin"
            shift 2
            ;;
        -v | --version)
            [ "$#" -ge 2 ] || die "--version requires a tag argument"
            VERSION="$2"
            shift 2
            ;;
        -v=* | --version=*)
            VERSION="${1#*=}"
            shift
            ;;
        *)
            die "unknown option: $1 (try --help)"
            ;;
    esac
done

say() {
    printf 'install.sh: %s\n' "$1"
}

os_name=
case "$(uname -s)" in
    Darwin) os_name="macos" ;;
    Linux) os_name="linux" ;;
    *MINGW* | *MSYS* | *CYGWIN*) os_name="windows" ;;
    *) die "unsupported operating system: $(uname -s) (build from source instead)" ;;
esac

arch=
case "$(uname -m)" in
    x86_64 | amd64) arch="x86_64" ;;
    aarch64 | arm64) arch="arm64" ;;
    *) die "unsupported architecture: $(uname -m) (build from source instead)" ;;
esac

target=
ext=
case "${os_name}:${arch}" in
    linux:x86_64) target="x86_64-unknown-linux-gnu" ; ext="tar.gz" ;;
    macos:x86_64) target="x86_64-apple-darwin" ; ext="tar.gz" ;;
    macos:arm64) target="aarch64-apple-darwin" ; ext="tar.gz" ;;
    windows:x86_64) target="x86_64-pc-windows-msvc" ; ext="zip" ;;
    *) die "no prebuilt dytype for ${os_name}/${arch} (build from source instead)" ;;
esac

if ! require_cmd curl && ! require_cmd wget; then
    die "need either 'curl' or 'wget' to download the release"
fi

sha_bin=
if require_cmd sha256sum; then
    sha_bin="sha256sum"
elif require_cmd shasum; then
    sha_bin="shasum -a 256"
else
    die "need either 'sha256sum' or 'shasum' to verify the download"
fi

# Resolve the latest release tag unless a version was pinned.
if [ -z "$VERSION" ]; then
    latest_url="https://api.github.com/repos/${REPO}/releases/latest"
    latest_json=""
    if require_cmd curl; then
        latest_json=$(curl -fsSL "$latest_url")
    else
        latest_json=$(wget -qO- "$latest_url")
    fi
    VERSION=$(printf '%s\n' "$latest_json" | sed -n 's/.*"tag_name" *: *"\([^"]*\)".*/\1/p' | head -n 1)
    [ -n "$VERSION" ] || die "could not detect the latest release; set DYTYPE_VERSION manually"
fi

case "$VERSION" in
    v*) VERSION_TAG="$VERSION" ;;
    *) VERSION_TAG="v${VERSION}" ;;
esac

ASSET="dytype-${VERSION_TAG}-${target}.${ext}"
ASSET_URL="${BASE_URL}/${VERSION_TAG}/${ASSET}"
SUM_URL="${BASE_URL}/${VERSION_TAG}/SHA256SUMS"

tmp=$(mktemp -d "${TMPDIR:-/tmp}/dytype-install.XXXXXX")
trap 'rm -rf "$tmp"' EXIT HUP INT TERM

say "target: ${os_name}/${arch} -> ${target} (${ext})"
say "fetching ${ASSET_URL} ..."
if require_cmd curl; then
    curl -fsSL -o "$tmp/archive" "$ASSET_URL" || die "download failed: ${ASSET_URL}"
    curl -fsSL -o "$tmp/SHA256SUMS" "$SUM_URL" || die "checksum download failed (are release assets public yet?): ${SUM_URL}"
else
    wget -q -O "$tmp/archive" "$ASSET_URL" || die "download failed: ${ASSET_URL}"
    wget -q -O "$tmp/SHA256SUMS" "$SUM_URL" || die "checksum download failed (are release assets public yet?): ${SUM_URL}"
fi

expected=$(awk -v asset="$ASSET" '$2 == asset { print $1; exit }' "$tmp/SHA256SUMS")
[ -n "$expected" ] || die "SHA256SUMS contains no entry for ${ASSET}"

actual=$($sha_bin "$tmp/archive" | awk '{print $1}')
[ "$actual" = "$expected" ] || die "checksum mismatch for ${ASSET} (download corrupt or tampered)"

if [ "$ext" = "zip" ]; then
    require_cmd unzip || die "need 'unzip' to extract ${ASSET}"
    (cd "$tmp" && unzip -o -q archive) || die "could not extract ${ASSET}"
    bin_name="dytype.exe"
else
    (cd "$tmp" && tar -xzf archive) || die "could not extract ${ASSET}"
    bin_name="dytype"
fi
[ -f "$tmp/$bin_name" ] || die "archive did not contain ${bin_name}"

mkdir -p "$INSTALL_DIR"
if require_cmd install; then
    install -m 755 "$tmp/$bin_name" "$INSTALL_DIR/"
else
    cp -f "$tmp/$bin_name" "$INSTALL_DIR/"
    chmod +x "$INSTALL_DIR/$bin_name"
fi

case ":$PATH:" in
    *":${INSTALL_DIR}:"*) ;;
    *) say "warning: ${INSTALL_DIR} is not on your PATH" ;;
esac

printf '\nInstalled dytype %s to %s\nRun \"%s\" in a terminal to start.\n' \
    "$VERSION_TAG" "$INSTALL_DIR/$bin_name" "$INSTALL_DIR/$bin_name"