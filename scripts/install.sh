#!/usr/bin/env bash
set -euo pipefail

REPO="${HOI_REPO:-kevinquillen/hoi}"
INSTALL_DIR="${INSTALL_DIR:-}"
VERSION="${HOI_VERSION:-}"

usage() {
  cat <<EOF
Install hoi from GitHub releases.

Usage: install.sh [options]

Options:
  -d, --dir DIR       Install directory (default: /usr/local/bin or ~/.local/bin)
  -v, --version VER   Install a specific version (default: latest release)
  -h, --help          Show this help

Environment:
  HOI_VERSION         Same as --version
  HOI_REPO            GitHub repository (default: kevinquillen/hoi)
  INSTALL_DIR         Same as --dir

Examples:
  curl -fsSL https://raw.githubusercontent.com/kevinquillen/hoi/main/scripts/install.sh | sh
  HOI_VERSION=0.7.1 ./scripts/install.sh --dir ~/.local/bin
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    -d | --dir)
      INSTALL_DIR="$2"
      shift 2
      ;;
    -v | --version)
      VERSION="$2"
      shift 2
      ;;
    -h | --help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if ! command -v uname >/dev/null 2>&1; then
  echo "Unable to detect platform: uname not found" >&2
  exit 1
fi

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux)
    PLATFORM="Linux-musl"
    ;;
  Darwin)
    PLATFORM="macOS"
    ;;
  *)
    echo "Unsupported operating system: $OS" >&2
    echo "Download a Windows build from https://github.com/${REPO}/releases" >&2
    exit 1
    ;;
esac

case "$ARCH" in
  x86_64 | amd64)
    ARCH_SUFFIX="x86_64"
    ;;
  arm64 | aarch64)
    ARCH_SUFFIX="arm64"
    ;;
  *)
    echo "Unsupported architecture: $ARCH" >&2
    exit 1
    ;;
esac

if [[ -z "$VERSION" ]]; then
  if ! command -v curl >/dev/null 2>&1; then
    echo "curl is required to resolve the latest release" >&2
    exit 1
  fi

  VERSION="$(
    curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
      | sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"v\(.*\)".*/\1/p' \
      | head -1
  )"

  if [[ -z "$VERSION" ]]; then
    echo "Unable to determine the latest release version" >&2
    exit 1
  fi
fi

VERSION="${VERSION#v}"
ARCHIVE="hoi-${PLATFORM}-${ARCH_SUFFIX}.tar.gz"
BASE_URL="https://github.com/${REPO}/releases/download/v${VERSION}"
CHECKSUM_URL="${BASE_URL}/${ARCHIVE}.sha256"

if [[ -z "$INSTALL_DIR" ]]; then
  if [[ -w /usr/local/bin ]]; then
    INSTALL_DIR="/usr/local/bin"
  else
    INSTALL_DIR="${HOME}/.local/bin"
  fi
fi

mkdir -p "$INSTALL_DIR"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

download() {
  local url="$1"
  local dest="$2"

  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$url" -o "$dest"
  elif command -v wget >/dev/null 2>&1; then
    wget -qO "$dest" "$url"
  else
    echo "curl or wget is required to download hoi" >&2
    exit 1
  fi
}

download "${BASE_URL}/${ARCHIVE}" "${TMP_DIR}/${ARCHIVE}"
download "$CHECKSUM_URL" "${TMP_DIR}/${ARCHIVE}.sha256"

EXPECTED_SHA256="$(awk '{print $1}' "${TMP_DIR}/${ARCHIVE}.sha256")"

if command -v sha256sum >/dev/null 2>&1; then
  ACTUAL_SHA256="$(sha256sum "${TMP_DIR}/${ARCHIVE}" | awk '{print $1}')"
elif command -v shasum >/dev/null 2>&1; then
  ACTUAL_SHA256="$(shasum -a 256 "${TMP_DIR}/${ARCHIVE}" | awk '{print $1}')"
else
  echo "sha256sum or shasum is required to verify the download" >&2
  exit 1
fi

if [[ "$EXPECTED_SHA256" != "$ACTUAL_SHA256" ]]; then
  echo "Checksum verification failed for ${ARCHIVE}" >&2
  exit 1
fi

tar xzf "${TMP_DIR}/${ARCHIVE}" -C "$TMP_DIR" hoi
install -m 0755 "${TMP_DIR}/hoi" "${INSTALL_DIR}/hoi"

echo "Installed hoi ${VERSION} to ${INSTALL_DIR}/hoi"

if ! command -v hoi >/dev/null 2>&1; then
  case ":$PATH:" in
    *":${INSTALL_DIR}:"*) ;;
    *)
      echo "Add ${INSTALL_DIR} to your PATH to run hoi"
      ;;
  esac
fi
