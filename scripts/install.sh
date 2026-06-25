#!/usr/bin/env bash
# Install or update Promptly to ~/.local (same layout as `make install-user`).
#
#   curl -fsSL https://raw.githubusercontent.com/mholtzhausen/promptly/main/scripts/install.sh | bash
#
# Optional environment variables:
#   PROMPTLY_VERSION   Install a specific release tag (e.g. v0.8.0) instead of latest.
#   PROMPTLY_INSTALL_REPO  GitHub "owner/repo" (default: mholtzhausen/promptly).
#   GITHUB_TOKEN       GitHub API token (avoids anonymous API rate limits).

set -euo pipefail

REPO="${PROMPTLY_INSTALL_REPO:-mholtzhausen/promptly}"
ARCH="$(uname -m)"
OS="$(uname -s)"

err() {
  printf 'promptly install: %s\n' "$*" >&2
}

die() {
  err "$@"
  exit 1
}

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || die "required command not found: $1"
}

github_api() {
  local url="https://api.github.com/repos/${REPO}/$1"
  if [[ -n "${GITHUB_TOKEN:-}" ]]; then
    curl -fsSL -H "Authorization: Bearer ${GITHUB_TOKEN}" -H "Accept: application/vnd.github+json" "$url"
  else
    curl -fsSL -H "Accept: application/vnd.github+json" "$url"
  fi
}

json_field() {
  # Extract the first JSON string value for a key without requiring jq.
  sed -n "s/.*\"$1\": \"\\([^\"]*\\)\".*/\\1/p" | head -n1
}

assert_linux_amd64() {
  [[ "$OS" == "Linux" ]] || die "unsupported operating system: $OS (Linux x86_64 only)"
  case "$ARCH" in
    x86_64 | amd64) ;;
    *) die "unsupported CPU architecture: $ARCH (only x86_64/amd64 is supported)" ;;
  esac
}

assert_not_root() {
  if [[ "$(id -u)" -eq 0 ]]; then
    die "refusing to install as root; rerun without sudo (installs to ~/.local)"
  fi
}

warn_runtime_deps() {
  if command -v ldconfig >/dev/null 2>&1; then
    if ! ldconfig -p 2>/dev/null | grep -q 'libwebkit2gtk-4\.1\.so'; then
      err "warning: libwebkit2gtk-4.1 does not appear to be installed."
      err "  Debian/Ubuntu: sudo apt-get install libwebkit2gtk-4.1-0 libgtk-4-1"
      err "  See docs/troubleshooting.md in the repository for other distributions."
    fi
  fi
}

resolve_release_tag() {
  if [[ -n "${PROMPTLY_VERSION:-}" ]]; then
    printf '%s\n' "$PROMPTLY_VERSION"
    return
  fi

  local tag
  tag="$(resolve_release_tag_redirect "$REPO" 2>/dev/null || true)"
  if [[ -n "$tag" ]]; then
    printf '%s\n' "$tag"
    return
  fi

  tag="$(resolve_release_tag_atom "$REPO" 2>/dev/null || true)"
  if [[ -n "$tag" ]]; then
    printf '%s\n' "$tag"
    return
  fi

  local release_json
  release_json="$(github_api releases/latest)"
  tag="$(printf '%s' "$release_json" | json_field tag_name)"
  [[ -n "$tag" ]] || die "could not determine latest release tag from GitHub"
  printf '%s\n' "$tag"
}

resolve_release_tag_redirect() {
  local repo="$1"
  local headers location tag
  headers="$(curl -fsSI -A "promptly-install" "https://github.com/${repo}/releases/latest")"
  location="$(printf '%s\n' "$headers" | awk 'tolower($1) == "location:" { print $2; exit }' | tr -d '\r')"
  [[ -n "$location" ]] || return 1
  case "$location" in
    */releases/tag/*)
      tag="${location##*/releases/tag/}"
      tag="${tag%%\?*}"
      tag="${tag%%#*}"
      tag="${tag//%2B/+}"
      printf '%s\n' "$tag"
      ;;
    *)
      return 1
      ;;
  esac
}

resolve_release_tag_atom() {
  local repo="$1"
  local atom title
  atom="$(curl -fsSL -A "promptly-install" "https://github.com/${repo}/releases.atom")"
  title="$(printf '%s' "$atom" | sed -n '/<entry>/,/<\/entry>/p' | sed -n 's:.*<title>\([^<]*\)</title>.*:\1:p' | head -n1)"
  [[ -n "$title" ]] || return 1
  printf '%s\n' "$title"
}

download_release_asset() {
  local tag="$1"
  local dest="$2"
  local asset_name="$3"
  local url checksum_url expected actual

  url="https://github.com/${REPO}/releases/download/${tag}/${asset_name}"
  if ! curl -fsSL -o "$dest" "$url" 2>/dev/null; then
    rm -f "$dest"
    return 1
  fi

  checksum_url="https://github.com/${REPO}/releases/download/${tag}/${asset_name}.sha256"
  if curl -fsSL -o "${dest}.sha256" "$checksum_url" 2>/dev/null; then
    expected="$(awk '{print $1}' "${dest}.sha256")"
    actual="$(sha256sum "$dest" | awk '{print $1}')"
    [[ "$expected" == "$actual" ]] || die "checksum mismatch for ${asset_name}"
  fi

  return 0
}

fetch_binary() {
  local tag="$1"
  local tmpdir="$2"
  local version="${tag#v}"
  local binary="${tmpdir}/promptly"
  local candidates=(
  "promptly-x86_64-linux"
  "promptly"
  "promptly-${version}-x86_64-linux.zip"
  )

  local candidate dest
  for candidate in "${candidates[@]}"; do
    dest="${tmpdir}/${candidate##*/}"
    if download_release_asset "$tag" "$dest" "$candidate"; then
      case "$candidate" in
        *.zip)
          require_cmd unzip
          unzip -p "$dest" promptly >"$binary"
          chmod 755 "$binary"
          ;;
        promptly-x86_64-linux)
          mv "$dest" "$binary"
          chmod 755 "$binary"
          ;;
        promptly)
          chmod 755 "$dest"
          ;;
        *)
          die "internal error: unknown asset type ${candidate}"
          ;;
      esac
      printf '%s\n' "$binary"
      return 0
    fi
  done

  die "no compatible Linux x86_64 release asset found for ${tag}"
}

fetch_packaging_file() {
  local tag="$1"
  local file="$2"
  local dest="$3"
  local url="https://raw.githubusercontent.com/${REPO}/${tag}/packaging/${file}"
  curl -fsSL -o "$dest" "$url"
}

install_layout() {
  local tag="$1"
  local binary="$2"
  local home="${HOME:?HOME is not set}"
  local bin_dir="${home}/.local/bin"
  local apps_dir="${home}/.local/share/applications"
  local autostart_dir="${home}/.config/autostart"
  local systemd_dir="${home}/.config/systemd/user"
  local desktop_src="${tmpdir}/promptly.desktop"
  local service_src="${tmpdir}/promptly.service"

  install -d "$bin_dir" "$apps_dir" "$autostart_dir" "$systemd_dir"
  install -m 755 "$binary" "${bin_dir}/promptly"

  fetch_packaging_file "$tag" "promptly.desktop" "$desktop_src"
  install -m 644 "$desktop_src" "${apps_dir}/promptly.desktop"
  sed "s|^Exec=promptly|Exec=${bin_dir}/promptly|" "$desktop_src" >"${autostart_dir}/promptly.desktop"
  chmod 644 "${autostart_dir}/promptly.desktop"

  fetch_packaging_file "$tag" "promptly.service" "$service_src"
  install -m 644 "$service_src" "${systemd_dir}/promptly.service"

  if command -v systemctl >/dev/null 2>&1 && systemctl --user show-environment >/dev/null 2>&1; then
    systemctl --user daemon-reload || true
  fi

  manage_systemd_service

  printf '\n'
  printf 'Installed promptly %s to %s\n' "${tag}" "${bin_dir}/promptly"
  if [[ "${PROMPTLY_MANAGE_SERVICE:-}" == "1" ]]; then
    printf 'Systemd user service managed automatically.\n'
  else
    printf 'Enable autostart: systemctl --user enable --now promptly.service\n'
  fi
  printf '\n'
}

manage_systemd_service() {
  if ! command -v systemctl >/dev/null 2>&1; then
    return 0
  fi
  if ! systemctl --user show-environment >/dev/null 2>&1; then
    return 0
  fi

  systemctl --user daemon-reload || true

  if [[ "${PROMPTLY_MANAGE_SERVICE:-}" == "1" ]]; then
    if systemctl --user is-active --quiet promptly.service 2>/dev/null; then
      systemctl --user restart promptly.service
    elif systemctl --user is-enabled --quiet promptly.service 2>/dev/null; then
      systemctl --user start promptly.service
    else
      systemctl --user enable --now promptly.service
    fi
    return 0
  fi

  if [[ -t 0 ]]; then
    printf 'Enable and start promptly.service for autostart? [y/N] '
    local reply
    IFS= read -r reply || reply=""
    case "$reply" in
      y | Y | yes | Yes | YES)
        systemctl --user enable --now promptly.service
        ;;
    esac
  fi
}

main() {
  assert_linux_amd64
  assert_not_root
  require_cmd curl
  require_cmd sha256sum
  require_cmd install
  require_cmd sed
  warn_runtime_deps

  local tag binary
  tag="$(resolve_release_tag)"
  tmpdir="$(mktemp -d)"
  trap 'rm -rf "$tmpdir"' EXIT

  binary="$(fetch_binary "$tag" "$tmpdir")"
  install_layout "$tag" "$binary"
}

main "$@"
