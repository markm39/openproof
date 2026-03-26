#!/usr/bin/env bash
set -euo pipefail

REPO="markm39/openproof"
BINARY_NAME="openproof"
INSTALL_DIR="${OPENPROOF_INSTALL_DIR:-/usr/local/bin}"

# Detect platform
OS="$(uname -s)"
ARCH="$(uname -m)"

case "${OS}" in
  Darwin) PLATFORM="apple-darwin" ;;
  Linux)  PLATFORM="unknown-linux-gnu" ;;
  *)
    echo "Error: unsupported OS '${OS}'. OpenProof supports macOS and Linux." >&2
    exit 1
    ;;
esac

case "${ARCH}" in
  arm64|aarch64) TARGET="aarch64-${PLATFORM}" ;;
  x86_64|x64)    TARGET="x86_64-${PLATFORM}" ;;
  *)
    echo "Error: unsupported architecture '${ARCH}'. OpenProof supports arm64 and x86_64." >&2
    exit 1
    ;;
esac

# Get version (from env or latest release)
if [ -n "${OPENPROOF_VERSION:-}" ]; then
  VERSION="${OPENPROOF_VERSION}"
else
  VERSION="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"v([^"]+)".*/\1/')"
  if [ -z "${VERSION}" ]; then
    echo "Error: could not determine latest version. Set OPENPROOF_VERSION manually." >&2
    exit 1
  fi
fi

TARBALL="openproof-v${VERSION}-${TARGET}.tar.gz"
URL="https://github.com/${REPO}/releases/download/v${VERSION}/${TARBALL}"

echo "Installing openproof v${VERSION} for ${TARGET}..."

TMPDIR="$(mktemp -d)"
trap 'rm -rf "${TMPDIR}"' EXIT

HTTP_CODE="$(curl -fsSL -w '%{http_code}' -o "${TMPDIR}/${TARBALL}" "${URL}" || true)"
if [ ! -f "${TMPDIR}/${TARBALL}" ] || [ "${HTTP_CODE}" != "200" ]; then
  echo "Error: failed to download ${URL}" >&2
  echo "Check that v${VERSION} exists at https://github.com/${REPO}/releases" >&2
  exit 1
fi

tar xzf "${TMPDIR}/${TARBALL}" -C "${TMPDIR}"

EXTRACTED_DIR="${TMPDIR}/openproof-v${VERSION}-${TARGET}"
if [ ! -f "${EXTRACTED_DIR}/${BINARY_NAME}" ]; then
  echo "Error: binary not found in tarball." >&2
  exit 1
fi

if [ -w "${INSTALL_DIR}" ]; then
  cp "${EXTRACTED_DIR}/${BINARY_NAME}" "${INSTALL_DIR}/"
else
  echo "Installing to ${INSTALL_DIR} requires sudo..."
  sudo cp "${EXTRACTED_DIR}/${BINARY_NAME}" "${INSTALL_DIR}/"
fi

chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
echo "Installed openproof to ${INSTALL_DIR}/${BINARY_NAME}"
