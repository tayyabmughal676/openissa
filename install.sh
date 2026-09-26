#!/usr/bin/env bash
set -euo pipefail

# OpenISSA Universal Installer Script
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/tayyabmughal676/openissa/main/install.sh | bash
#   curl -fsSL https://raw.githubusercontent.com/tayyabmughal676/openissa/main/install.sh | bash -s -- --version v0.1.0

REPO="tayyabmughal676/openissa"
BINARY_NAME="openissa"
TARGET_DIR="${HOME}/.local/bin"
VERSION=""

# Parse arguments
while [[ $# -gt 0 ]]; do
  case "$1" in
    --version|-v)
      VERSION="$2"
      shift 2
      ;;
    --dir|-d)
      TARGET_DIR="$2"
      shift 2
      ;;
    *)
      echo "Unknown option: $1"
      echo "Usage: $0 [--version vX.Y.Z] [--dir /install/path]"
      exit 1
      ;;
  esac
done

echo "==> OpenISSA Installer"

# Detect OS
OS="$(uname -s)"
case "${OS}" in
  Darwin)
    OS_TYPE="apple-darwin"
    ;;
  Linux)
    OS_TYPE="unknown-linux-gnu"
    ;;
  *)
    echo "Error: Unsupported operating system: ${OS}"
    echo "Windows users can download pre-compiled .zip binaries directly from:"
    echo "https://github.com/${REPO}/releases"
    exit 1
    ;;
esac

# Detect Architecture
ARCH="$(uname -m)"
case "${ARCH}" in
  x86_64|amd64)
    ARCH_TYPE="x86_64"
    ;;
  arm64|aarch64)
    ARCH_TYPE="aarch64"
    ;;
  *)
    echo "Error: Unsupported architecture: ${ARCH}"
    exit 1
    ;;
esac

TARGET="${ARCH_TYPE}-${OS_TYPE}"

# Determine version
if [[ -z "${VERSION}" ]]; then
  echo "==> Fetching latest release version from GitHub..."
  VERSION=$(curl -sSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
  if [[ -z "${VERSION}" ]]; then
    echo "Error: Failed to determine latest version. Please specify --version vX.Y.Z manually."
    exit 1
  fi
fi

echo "==> Target Architecture: ${TARGET}"
echo "==> Selected Version:    ${VERSION}"

ARCHIVE_NAME="${BINARY_NAME}-${VERSION}-${TARGET}.tar.gz"
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/${ARCHIVE_NAME}"
CHECKSUM_URL="https://github.com/${REPO}/releases/download/${VERSION}/${ARCHIVE_NAME}.sha256"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "${TMP_DIR}"' EXIT

echo "==> Downloading ${ARCHIVE_NAME}..."
curl -sSL --fail "${DOWNLOAD_URL}" -o "${TMP_DIR}/${ARCHIVE_NAME}"

# Verify Checksum if available
if curl -sSL --fail "${CHECKSUM_URL}" -o "${TMP_DIR}/${ARCHIVE_NAME}.sha256" 2>/dev/null; then
  echo "==> Verifying SHA256 checksum..."
  cd "${TMP_DIR}"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum -c "${ARCHIVE_NAME}.sha256"
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 -c "${ARCHIVE_NAME}.sha256"
  fi
  cd - >/dev/null
fi

# Extract and install
echo "==> Installing ${BINARY_NAME} to ${TARGET_DIR}..."
mkdir -p "${TARGET_DIR}"
tar -xzf "${TMP_DIR}/${ARCHIVE_NAME}" -C "${TMP_DIR}"
mv "${TMP_DIR}/${BINARY_NAME}" "${TARGET_DIR}/${BINARY_NAME}"
chmod +x "${TARGET_DIR}/${BINARY_NAME}"

echo "==> Installation complete!"
echo "    Installed binary: ${TARGET_DIR}/${BINARY_NAME}"

# PATH warning if needed
if [[ ":$PATH:" != *":${TARGET_DIR}:"* ]]; then
  echo ""
  echo "Notice: ${TARGET_DIR} is not currently in your \$PATH."
  echo "Add it by running:"
  echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
  echo "And add that line to your ~/.bashrc or ~/.zshrc."
fi

echo ""
echo "To test OpenISSA, run:"
echo "  ${BINARY_NAME} --version"
echo ""
echo "To connect to Claude Code:"
echo "  claude mcp add openissa -- openissa mcp"
