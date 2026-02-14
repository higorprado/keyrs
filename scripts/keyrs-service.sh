#!/usr/bin/env bash
set -euo pipefail

# ============================================================
# Configuration
# ============================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
SCRIPT_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"

SERVICE_NAME="keyrs.service"
SERVICE_DIR="${HOME}/.config/systemd/user"
SERVICE_PATH="${SERVICE_DIR}/${SERVICE_NAME}"
UDEV_RULES_TARGET="/etc/udev/rules.d/99-keyrs.rules"
CONFIG_DIR="${HOME}/.config/keyrs"
CONFIG_SOURCE_DIR="${REPO_ROOT}/config.d.example"
CONFIG_COMPOSE_DIR="${CONFIG_DIR}/config.d"
CONFIG_UDEV_RULES="${CONFIG_DIR}/keyrs-udev.rules"
PROFILES_DIR="${REPO_ROOT}/profiles"
BUILTIN_PROFILES_DIR="${CONFIG_DIR}/profiles"  # Installed profiles for runtime mode
BIN_DIR="${HOME}/.local/bin"
TARGET_BIN="${BIN_DIR}/keyrs"
TARGET_TUI_BIN="${BIN_DIR}/keyrs-tui"
RUNTIME_CTL="${BIN_DIR}/keyrs-service"
PROFILE_CACHE_DIR="${CONFIG_DIR}/profile-cache"
BACKUPS_DIR="${CONFIG_DIR}/backups"

SYSTEMCTL_BIN="${SYSTEMCTL_BIN:-systemctl}"
UDEVADM_BIN="${UDEVADM_BIN:-udevadm}"

# Flags
DRY_RUN=false
FORCE=false
ASSUME_YES=false
QUIET=false
COLOR_MODE="auto"  # auto, always, never

# Arguments
BIN_SOURCE=""
TUI_BIN_SOURCE=""
COMPOSE_SOURCE_DIR=""
SELECTED_PROFILE=""
PROFILE_URL=""

# Temp files for cleanup
declare -a TEMP_FILES=()

RUNTIME_ONLY=false
if [[ "${KEYRS_RUNTIME_ONLY:-0}" == "1" || "${SCRIPT_PATH}" == "${RUNTIME_CTL}" ]]; then
  RUNTIME_ONLY=true
fi

# ============================================================
# Colors (--color=always/never/auto, respects NO_COLOR)
# ============================================================

setup_colors() {
  case "${COLOR_MODE}" in
    never)
      COLOR_RESET=""
      COLOR_RED=""
      COLOR_GREEN=""
      COLOR_YELLOW=""
      COLOR_BLUE=""
      COLOR_BOLD=""
      return
      ;;
    always)
      # Force colors on
      ;;
    auto|"")
      # Auto-detect: check if EITHER stdout or stderr is a terminal
      # Also respect NO_COLOR environment variable
      if [[ -n "${NO_COLOR:-}" ]] || ! { [[ -t 1 ]] || [[ -t 2 ]]; }; then
        COLOR_RESET=""
        COLOR_RED=""
        COLOR_GREEN=""
        COLOR_YELLOW=""
        COLOR_BLUE=""
        COLOR_BOLD=""
        return
      fi
      ;;
  esac

  # Get colors via tput
  COLOR_RESET="$(tput sgr0 2>/dev/null || printf '\033[0m')"
  COLOR_RED="$(tput setaf 1 2>/dev/null || printf '\033[31m')"
  COLOR_GREEN="$(tput setaf 2 2>/dev/null || printf '\033[32m')"
  COLOR_YELLOW="$(tput setaf 3 2>/dev/null || printf '\033[33m')"
  COLOR_BLUE="$(tput setaf 4 2>/dev/null || printf '\033[34m')"
  COLOR_BOLD="$(tput bold 2>/dev/null || printf '\033[1m')"
}

# Initialize colors (will be set after parsing --color option)
COLOR_RESET=""
COLOR_RED=""
COLOR_GREEN=""
COLOR_YELLOW=""
COLOR_BLUE=""
COLOR_BOLD=""

# ============================================================
# Cleanup and Signal Handling
# ============================================================

cleanup() {
  for f in "${TEMP_FILES[@]}"; do
    [[ -f "${f}" ]] && rm -f "${f}"
  done
}

trap cleanup EXIT
trap 'log "Interrupted"; exit 130' INT
trap 'log "Terminated"; exit 143' TERM

# ============================================================
# Logging Functions
# ============================================================

log() {
  ${QUIET} || printf '[keyrs-service] %s\n' "$*"
}

log_info() {
  ${QUIET} || printf '%s\n' "${COLOR_BLUE}ℹ${COLOR_RESET} $*"
}

log_success() {
  ${QUIET} || printf '%s\n' "${COLOR_GREEN}✓${COLOR_RESET} $*"
}

log_warn() {
  ${QUIET} || printf '%s\n' "${COLOR_YELLOW}⚠${COLOR_RESET} $*" >&2
}

log_error() {
  printf '%s\n' "${COLOR_RED}✗${COLOR_RESET} $*" >&2
}

run() {
  if ${DRY_RUN}; then
    printf '[dry-run] %s\n' "$*"
    return 0
  fi
  "$@"
}

usage() {
  local header
  local examples
  if ${RUNTIME_ONLY}; then
    header="Usage:
  keyrs-service <command> [options]"
    examples="Examples:
  keyrs-service apply-config
  keyrs-service profile-set developer
  keyrs-service restart"
  else
    header="Usage:
  scripts/keyrs-service.sh <command> [options]"
    examples="Examples:
  scripts/keyrs-service.sh install
  scripts/keyrs-service.sh install --profile developer
  scripts/keyrs-service.sh install-udev"
  fi

  cat <<USAGE
${header}

Commands:
USAGE

  if ! ${RUNTIME_ONLY}; then
    cat <<USAGE
  install          Install keyrs binary, profiles, and service
  uninstall        Remove service and installed files
USAGE
  fi

  cat <<USAGE

Service:
  start            Start the keyrs service
  stop             Stop the keyrs service
  restart          Restart the keyrs service
  status           Show service status

Configuration:
  apply-config     Compose and validate config, restart service

Profiles:
  list-profiles    List available profiles
  show-profile     Show profile details: show-profile <name>
  profile-set      Set active profile: profile-set <name> [--url <url>]
  profile-select   Interactively select a profile
USAGE

  cat <<USAGE

Advanced:
  install-udev     Install udev rules (requires sudo)
  uninstall-udev   Remove udev rules (requires sudo)

Options:
  --bin <path>         Binary source path (default: ./target/release/keyrs)
  --source-dir <path>  Source dir for apply-config (default: ~/.config/keyrs/config.d)
  --profile <name>     Profile name to use
  --url <url>          Profile URL for profile-set
  --force              Overwrite existing config files
  --yes                Skip confirmation prompt
  --quiet, -q          Suppress output (only errors)
  --color <mode>       Color output: auto, always, never (default: auto)
  --dry-run            Print actions without executing
  -h, --help           Show this help

${examples}
USAGE
}

ensure_systemctl_user() {
  if ! command -v "${SYSTEMCTL_BIN}" >/dev/null 2>&1; then
    log "systemctl not found: ${SYSTEMCTL_BIN}"
    exit 1
  fi
}

ensure_udevadm() {
  if ! command -v "${UDEVADM_BIN}" >/dev/null 2>&1; then
    log "udevadm not found: ${UDEVADM_BIN}"
    exit 1
  fi
}

parse_args() {
  # Parse global options first (before command)
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --quiet|-q)
        QUIET=true
        shift
        ;;
      --color)
        COLOR_MODE="${2:-auto}"
        shift 2
        ;;
      -h|--help)
        usage
        exit 0
        ;;
      *)
        break
        ;;
    esac
  done

  COMMAND="${1:-}"
  if [[ -z "${COMMAND}" ]]; then
    usage
    exit 1
  fi
  shift || true

  # Handle positional argument for profile commands
  if [[ -n "${1:-}" ]] && [[ ! "${1}" =~ ^- ]]; then
    case "${COMMAND}" in
      show-profile|profile-set)
        SELECTED_PROFILE="${1}"
        shift
        ;;
    esac
  fi

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --bin)
        BIN_SOURCE="${2:-}"
        shift 2
        ;;
      --force)
        FORCE=true
        shift
        ;;
      --source-dir)
        COMPOSE_SOURCE_DIR="${2:-}"
        shift 2
        ;;
      --profile)
        SELECTED_PROFILE="${2:-}"
        shift 2
        ;;
      --url|--profile-url)
        PROFILE_URL="${2:-}"
        shift 2
        ;;
      --yes)
        ASSUME_YES=true
        shift
        ;;
      --quiet|-q)
        QUIET=true
        shift
        ;;
      --color)
        COLOR_MODE="${2:-auto}"
        shift 2
        ;;
      --dry-run)
        DRY_RUN=true
        shift
        ;;
      -h|--help)
        usage
        exit 0
        ;;
      *)
        log "Unknown option: $1"
        usage
        exit 1
        ;;
    esac
  done
}

resolve_bin_source() {
  if [[ -n "${BIN_SOURCE}" ]]; then
    return
  fi
  if [[ -x "${REPO_ROOT}/target/release/keyrs" ]]; then
    BIN_SOURCE="${REPO_ROOT}/target/release/keyrs"
    return
  fi
  if command -v keyrs >/dev/null 2>&1; then
    BIN_SOURCE="$(command -v keyrs)"
    return
  fi

  log "Could not resolve binary source. Build first: cargo build --release --features pure-rust --bin keyrs"
  exit 1
}

resolve_tui_bin_source() {
  if [[ -n "${TUI_BIN_SOURCE}" ]]; then
    return
  fi
  if [[ -x "${REPO_ROOT}/target/release/keyrs-tui" ]]; then
    TUI_BIN_SOURCE="${REPO_ROOT}/target/release/keyrs-tui"
    return
  fi
  if command -v keyrs-tui >/dev/null 2>&1; then
    TUI_BIN_SOURCE="$(command -v keyrs-tui)"
    return
  fi
  TUI_BIN_SOURCE=""
}

resolve_runtime_bin() {
  if [[ -x "${TARGET_BIN}" ]]; then
    return
  fi
  if command -v keyrs >/dev/null 2>&1; then
    TARGET_BIN="$(command -v keyrs)"
    return
  fi
  if [[ -x "${REPO_ROOT}/target/release/keyrs" ]]; then
    TARGET_BIN="${REPO_ROOT}/target/release/keyrs"
    return
  fi
  log "Could not resolve keyrs binary. Install first or put keyrs in PATH."
  exit 1
}

resolve_udev_rules_source() {
  if [[ -f "${CONFIG_UDEV_RULES}" ]]; then
    echo "${CONFIG_UDEV_RULES}"
    return
  fi
  if [[ -f "${REPO_ROOT}/dist/keyrs-udev.rules" ]]; then
    echo "${REPO_ROOT}/dist/keyrs-udev.rules"
    return
  fi
  log "Missing udev rules source (checked ${CONFIG_UDEV_RULES} and ${REPO_ROOT}/dist/keyrs-udev.rules)"
  exit 1
}

udev_rules_installed() {
  [[ -f "${UDEV_RULES_TARGET}" ]]
}

prompt_udev_install() {
  if ${DRY_RUN}; then
    log "Dry-run: would prompt for udev installation"
    return
  fi

  if udev_rules_installed; then
    log "udev rules already installed at ${UDEV_RULES_TARGET}"
    return
  fi

  echo ""
  echo "Install udev rules for keyboard device access?"
  echo ""
  echo "  This allows keyrs to access keyboard devices without sudo."
  echo "  Requires root password for installation."
  echo "  Recommended once per machine."
  echo ""

  local response=""
  if ${ASSUME_YES}; then
    response="y"
  else
    read -r -p "Install udev rules? [y/N] " response
  fi

  if [[ "${response}" =~ ^[Yy]$ ]]; then
    log "Installing udev rules..."
    local udev_rules_source
    udev_rules_source="$(resolve_udev_rules_source)"

    if ! run_privileged install -D -m 0644 "${udev_rules_source}" "${UDEV_RULES_TARGET}"; then
      log "Failed to install udev rules (password incorrect or insufficient privileges)"
      log "You can install manually later with: keyrs-service install-udev"
      return
    fi

    run_privileged "${UDEVADM_BIN}" control --reload-rules
    run_privileged "${UDEVADM_BIN}" trigger --subsystem-match=input
    log "udev rules installed successfully"
    log "Re-login if keyboard permissions are still denied."
  else
    log "Skipping udev installation"
    log "Install manually later with: keyrs-service install-udev"
  fi
}

confirm_or_abort() {
  local prompt_title="$1"
  local details="$2"

  if ${ASSUME_YES}; then
    log "Confirmation skipped (--yes)"
    return
  fi

  cat <<EOF

${prompt_title}
${details}

Proceed? [y/N]
EOF

  local answer
  read -r answer
  case "${answer}" in
    y|Y|yes|YES)
      ;;
    *)
      log "Aborted by user."
      exit 1
      ;;
  esac
}

run_privileged() {
  if ${DRY_RUN}; then
    printf '[dry-run] %s\n' "$*"
    return 0
  fi

  if [[ "${EUID}" -eq 0 ]]; then
    "$@"
    return 0
  fi

  if command -v sudo >/dev/null 2>&1; then
    sudo "$@"
    return 0
  fi

  log "This action requires root privileges (run as root or install sudo)."
  exit 1
}

write_service_file() {
  local tmp
  tmp="$(mktemp)"
  TEMP_FILES+=("${tmp}")
  cat > "${tmp}" <<UNIT
[Unit]
Description=keyrs keyboard remapper
After=graphical-session.target
Wants=graphical-session.target

[Service]
Type=simple
ExecStart=%h/.local/bin/keyrs --config %h/.config/keyrs/config.toml
Restart=on-failure
RestartSec=2
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=default.target
UNIT

  if ${DRY_RUN}; then
    log "Would write service file to ${SERVICE_PATH}"
    rm -f "${tmp}"
    return
  fi

  if [[ -f "${SERVICE_PATH}" ]]; then
    cp "${SERVICE_PATH}" "${SERVICE_PATH}.bak"
    log "Backed up existing service to ${SERVICE_PATH}.bak"
  fi
  mv "${tmp}" "${SERVICE_PATH}"
}

select_profile_interactive() {
  local search_dirs
  search_dirs="$(get_profiles_search_dirs)"

  if [[ -z "${search_dirs}" ]]; then
    return 1
  fi

  local profiles=()
  local descriptions=()
  local seen_names=""

  for dir in ${search_dirs}; do
    for profile_path in "${dir}"/*/profile.toml; do
      if [[ -f "${profile_path}" ]]; then
        local profile_dir
        profile_dir="$(dirname "${profile_path}")"
        local profile_name
        profile_name="$(basename "${profile_dir}")"

        # Skip duplicates
        if [[ "${seen_names}" == *":${profile_name}:"* ]]; then
          continue
        fi
        seen_names="${seen_names}:${profile_name}:"

        profiles+=("${profile_name}")

        local description=""
        if command -v grep >/dev/null 2>&1; then
          description="$(grep -E '^description[[:space:]]*=' "${profile_path}" 2>/dev/null | head -1 | sed 's/^description[[:space:]]*=[[:space:]]*//; s/"//g' || true)"
        fi
        descriptions+=("${description:-No description}")
      fi
    done
  done

  if [[ ${#profiles[@]} -eq 0 ]]; then
    return 1
  fi

  echo ""
  echo "Select a configuration profile:"
  echo ""
  for i in "${!profiles[@]}"; do
    printf "  %2d) %-18s %s\n" $((i+1)) "${profiles[$i]}" "${descriptions[$i]}"
  done
  echo ""

  local selection=""
  local selected_name=""

  while true; do
    if ${ASSUME_YES}; then
      selection="1"
    else
      read -r -p "Enter number (1-${#profiles[@]}) or press Enter for default [mac-standard]: " selection
    fi

    if [[ -z "${selection}" ]]; then
      selected_name="mac-standard"
      break
    fi

    if [[ "${selection}" =~ ^[0-9]+$ ]] && [[ "${selection}" -ge 1 ]] && [[ "${selection}" -le ${#profiles[@]} ]]; then
      selected_name="${profiles[$((selection-1))]}"
      break
    fi

    echo "Invalid selection. Please enter a number between 1 and ${#profiles[@]}."
  done

  SELECTED_PROFILE="${selected_name}"
  log "Selected profile: ${SELECTED_PROFILE}"
  return 0
}

install_cmd() {
  ensure_systemctl_user
  if ${RUNTIME_ONLY}; then
    log "install is unavailable in runtime mode."
    exit 1
  fi
  resolve_bin_source
  resolve_tui_bin_source

  local profile_source_dir=""

  if [[ -n "${PROFILE_URL:-}" ]]; then
    local cache_dir="${PROFILE_CACHE_DIR}/install-$(date +%s)"
    if ! download_profile "${PROFILE_URL}" "${cache_dir}"; then
      log "Failed to download profile from URL"
      exit 1
    fi
    profile_source_dir="${cache_dir}"
    SELECTED_PROFILE="custom URL"
  elif [[ -n "${SELECTED_PROFILE:-}" ]]; then
    profile_source_dir="$(get_profile_dir "${SELECTED_PROFILE}")"
    if [[ ! -f "${profile_source_dir}/profile.toml" ]]; then
      log "Profile not found: ${SELECTED_PROFILE}"
      log "Run 'keyrs-service list-profiles' to see available profiles."
      exit 1
    fi
  elif select_profile_interactive; then
    profile_source_dir="$(get_profile_dir "${SELECTED_PROFILE}")"
    if [[ ! -f "${profile_source_dir}/profile.toml" ]]; then
      log "Profile not found: ${SELECTED_PROFILE}"
      exit 1
    fi
  else
    log "No profiles available; using example config"
  fi

  local config_source_display="${CONFIG_SOURCE_DIR}"
  if [[ -n "${profile_source_dir}" ]]; then
    config_source_display="${profile_source_dir}/config.d (profile: ${SELECTED_PROFILE:-custom URL})"
  fi

  confirm_or_abort \
    "About to install and activate keyrs service:" \
    "  - Binary source: ${BIN_SOURCE}
  - Install binary: ${TARGET_BIN}
  - Install TUI binary: ${TARGET_TUI_BIN}
  - Config fragments source: ${config_source_display}
  - Config fragments target: ${CONFIG_COMPOSE_DIR}
  - Compose output: ${CONFIG_DIR}/config.toml
  - Settings target: ${CONFIG_DIR}/settings.toml
  - Service file: ${SERVICE_PATH}
  - Will run: systemctl --user daemon-reload
  - Will run: systemctl --user enable --now ${SERVICE_NAME}
  - Existing configs are preserved unless --force is used."

  log "Installing keyrs service"
  run mkdir -p "${BIN_DIR}" "${CONFIG_DIR}" "${SERVICE_DIR}"
  run install -m 755 "${BIN_SOURCE}" "${TARGET_BIN}"
  if [[ -n "${TUI_BIN_SOURCE}" ]]; then
    run install -m 755 "${TUI_BIN_SOURCE}" "${TARGET_TUI_BIN}"
  else
    log "keyrs-tui binary not found; skipping ${TARGET_TUI_BIN} install"
  fi
  run install -m 755 "${SCRIPT_PATH}" "${RUNTIME_CTL}"

  local effective_config_source="${CONFIG_SOURCE_DIR}"
  if [[ -n "${profile_source_dir}" ]]; then
    effective_config_source="${profile_source_dir}/config.d"
  fi

  if [[ ! -d "${effective_config_source}" ]]; then
    log "Missing config source at ${effective_config_source}"
    exit 1
  fi

  if [[ -d "${CONFIG_COMPOSE_DIR}" && "${FORCE}" != true && -n "${profile_source_dir}" ]]; then
    local backup_dir="${CONFIG_DIR}/config.d.bak.$(date +%Y%m%d%H%M%S)"
    run mv "${CONFIG_COMPOSE_DIR}" "${backup_dir}"
    log "Backed up existing config to ${backup_dir}"
  fi

  if [[ ! -d "${CONFIG_COMPOSE_DIR}" || "${FORCE}" == true || -n "${profile_source_dir}" ]]; then
    run mkdir -p "${CONFIG_COMPOSE_DIR}"
    run cp -a "${effective_config_source}/." "${CONFIG_COMPOSE_DIR}/"
    log "Installed config fragments into ${CONFIG_COMPOSE_DIR}"
  else
    log "Keeping existing ${CONFIG_COMPOSE_DIR}"
  fi

  if [[ ! -f "${CONFIG_DIR}/settings.toml" || "${FORCE}" == true ]]; then
    run cp "${REPO_ROOT}/settings.toml" "${CONFIG_DIR}/settings.toml"
    log "Installed settings.toml"
  else
    log "Keeping existing ${CONFIG_DIR}/settings.toml"
  fi

  if [[ -f "${REPO_ROOT}/dist/keyrs-udev.rules" ]]; then
    run cp "${REPO_ROOT}/dist/keyrs-udev.rules" "${CONFIG_UDEV_RULES}"
    log "Installed ${CONFIG_UDEV_RULES}"
  fi

  # Install built-in profiles for runtime use
  if [[ -d "${PROFILES_DIR}" ]]; then
    run mkdir -p "${BUILTIN_PROFILES_DIR}"
    run cp -a "${PROFILES_DIR}/." "${BUILTIN_PROFILES_DIR}/"
    log "Installed profiles to ${BUILTIN_PROFILES_DIR}"
  fi

  run "${TARGET_BIN}" --compose-config "${CONFIG_COMPOSE_DIR}" --compose-output "${CONFIG_DIR}/config.toml"
  run "${TARGET_BIN}" --check-config --config "${CONFIG_DIR}/config.toml"

  write_service_file
  run "${SYSTEMCTL_BIN}" --user daemon-reload
  run "${SYSTEMCTL_BIN}" --user enable --now "${SERVICE_NAME}"
  run "${SYSTEMCTL_BIN}" --user --no-pager --full status "${SERVICE_NAME}"

  log_success "Install complete"

  prompt_udev_install
}

uninstall_cmd() {
  ensure_systemctl_user

  local remove_binaries=false
  local remove_config=false
  local remove_udev=false

  # Check what exists
  local has_bin=false has_config=false has_udev=false
  [[ -f "${TARGET_BIN}" ]] && has_bin=true
  [[ -d "${CONFIG_DIR}" ]] && has_config=true
  [[ -f "${UDEV_RULES_TARGET}" ]] && has_udev=true

  echo ""
  echo "${COLOR_YELLOW}About to uninstall keyrs service integration:${COLOR_RESET}"
  echo "  - Will stop and disable: ${SERVICE_NAME}"
  echo "  - Will remove service file: ${SERVICE_PATH}"
  echo ""

  # Prompt for binaries
  if ${has_bin}; then
    echo "Also remove installed binaries?"
    echo "  - ${TARGET_BIN}"
    if [[ -f "${TARGET_TUI_BIN}" ]]; then
      echo "  - ${TARGET_TUI_BIN}"
    fi
    echo "  - ${RUNTIME_CTL}"
    echo ""

    local response=""
    if ${ASSUME_YES}; then
      response="n"
    else
      read -r -p "Remove binaries? [y/N] " response
    fi
    if [[ "${response}" =~ ^[Yy]$ ]]; then
      remove_binaries=true
    fi
    echo ""
  fi

  # Prompt for config
  if ${has_config}; then
    echo "${COLOR_YELLOW}Also remove configuration files?${COLOR_RESET}"
    echo "  - ${CONFIG_DIR}/"
    echo ""
    echo "  ${COLOR_RED}WARNING: This will permanently delete all your keyrs configurations,${COLOR_RESET}"
    echo "  ${COLOR_RED}         profiles, backups, and settings.${COLOR_RESET}"
    echo ""

    local response=""
    if ${ASSUME_YES}; then
      response="n"
    else
      read -r -p "Remove config files? [y/N] " response
    fi
    if [[ "${response}" =~ ^[Yy]$ ]]; then
      remove_config=true
    fi
    echo ""
  fi

  # Prompt for udev
  if ${has_udev}; then
    echo "Also remove udev rules?"
    echo "  - ${UDEV_RULES_TARGET}"
    echo ""
    echo "  ${COLOR_BLUE}Note: This requires root privileges (sudo).${COLOR_RESET}"
    echo ""

    local response=""
    if ${ASSUME_YES}; then
      response="n"
    else
      read -r -p "Remove udev rules? [y/N] " response
    fi
    if [[ "${response}" =~ ^[Yy]$ ]]; then
      remove_udev=true
    fi
    echo ""
  fi

  # Summary
  echo "${COLOR_BOLD}Summary:${COLOR_RESET}"
  echo "  - Stop and disable service: yes"
  echo "  - Remove service file: yes"
  echo "  - Remove binaries: $(${remove_binaries} && echo "yes" || echo "no")"
  echo "  - Remove config files: $(${remove_config} && echo "yes" || echo "no")"
  echo "  - Remove udev rules: $(${remove_udev} && echo "yes" || echo "no")"
  echo ""

  local confirm=""
  if ${ASSUME_YES}; then
    confirm="y"
  else
    read -r -p "Proceed? [y/N] " confirm
  fi

  if [[ ! "${confirm}" =~ ^[Yy]$ ]]; then
    log "Aborted by user."
    exit 1
  fi

  log "Uninstalling keyrs..."

  # Stop and disable service
  run "${SYSTEMCTL_BIN}" --user disable --now "${SERVICE_NAME}" || true

  # Remove service file
  if [[ -f "${SERVICE_PATH}" ]]; then
    run rm -f "${SERVICE_PATH}"
    log "Removed ${SERVICE_PATH}"
  fi
  run "${SYSTEMCTL_BIN}" --user daemon-reload

  # Remove binaries
  if ${remove_binaries}; then
    if [[ -f "${TARGET_BIN}" ]]; then
      run rm -f "${TARGET_BIN}"
      log "Removed ${TARGET_BIN}"
    fi
    if [[ -f "${TARGET_TUI_BIN}" ]]; then
      run rm -f "${TARGET_TUI_BIN}"
      log "Removed ${TARGET_TUI_BIN}"
    fi
    if [[ -f "${RUNTIME_CTL}" ]]; then
      run rm -f "${RUNTIME_CTL}"
      log "Removed ${RUNTIME_CTL}"
    fi
  fi

  # Remove config
  if ${remove_config}; then
    if [[ -d "${CONFIG_DIR}" ]]; then
      run rm -rf "${CONFIG_DIR}"
      log "Removed ${CONFIG_DIR}"
    fi
  fi

  # Remove udev rules
  if ${remove_udev}; then
    if [[ -f "${UDEV_RULES_TARGET}" ]]; then
      run_privileged rm -f "${UDEV_RULES_TARGET}"
      run_privileged "${UDEVADM_BIN}" control --reload-rules
      run_privileged "${UDEVADM_BIN}" trigger --subsystem-match=input
      log "Removed ${UDEV_RULES_TARGET}"
    fi
  fi

  log_success "Uninstall complete"
  if ! ${remove_binaries} || ! ${remove_config}; then
    echo ""
    log "To fully remove later:"
    ${has_bin} && ! ${remove_binaries} && echo "  rm -f ${TARGET_BIN} ${TARGET_TUI_BIN} ${RUNTIME_CTL}"
    ${has_config} && ! ${remove_config} && echo "  rm -rf ${CONFIG_DIR}"
  fi
}

apply_config_cmd() {
  ensure_systemctl_user
  resolve_runtime_bin
  run mkdir -p "${CONFIG_DIR}"

  local source_dir="${COMPOSE_SOURCE_DIR:-${CONFIG_COMPOSE_DIR}}"
  local output_path="${CONFIG_DIR}/config.toml"
  local tmp_path
  tmp_path="$(mktemp "${CONFIG_DIR}/config.toml.new.XXXXXX")"
  TEMP_FILES+=("${tmp_path}")

  if [[ ! -d "${source_dir}" ]]; then
    log "Source config directory not found: ${source_dir}"
    exit 1
  fi

  confirm_or_abort \
    "About to apply new keyrs config:" \
    "  - Source fragments: ${source_dir}
  - Temporary output: ${tmp_path}
  - Final output: ${output_path}
  - Will run: keyrs --compose-config --compose-output
  - Will run: keyrs --check-config
  - Will replace ${output_path} only if validation succeeds
  - Will restart: ${SERVICE_NAME}"

  run "${TARGET_BIN}" --compose-config "${source_dir}" --compose-output "${tmp_path}"
  run "${TARGET_BIN}" --check-config --config "${tmp_path}"

  if ${DRY_RUN}; then
    log "Dry-run complete; no files changed."
    return
  fi

  if [[ -f "${output_path}" ]]; then
    cp "${output_path}" "${output_path}.bak"
    log "Backed up existing config to ${output_path}.bak"
  fi
  mv "${tmp_path}" "${output_path}"
  log "Updated ${output_path}"

  run "${SYSTEMCTL_BIN}" --user restart "${SERVICE_NAME}"
  run "${SYSTEMCTL_BIN}" --user --no-pager --full status "${SERVICE_NAME}"
}

install_udev_cmd() {
  ensure_udevadm
  local udev_rules_source
  udev_rules_source="$(resolve_udev_rules_source)"

  confirm_or_abort \
    "About to install keyrs udev rules:" \
    "  - Rules source: ${udev_rules_source}
  - Rules target: ${UDEV_RULES_TARGET}
  - Will run: udevadm control --reload-rules
  - Will run: udevadm trigger --subsystem-match=input
  - Root privileges are required."

  log "Installing keyrs udev rules"
  run_privileged install -D -m 0644 "${udev_rules_source}" "${UDEV_RULES_TARGET}"
  run_privileged "${UDEVADM_BIN}" control --reload-rules
  run_privileged "${UDEVADM_BIN}" trigger --subsystem-match=input
  log "udev rules installed. Re-login if permissions are still denied."
}

uninstall_udev_cmd() {
  ensure_udevadm

  confirm_or_abort \
    "About to remove keyrs udev rules:" \
    "  - Rules target: ${UDEV_RULES_TARGET}
  - Will run: udevadm control --reload-rules
  - Will run: udevadm trigger --subsystem-match=input
  - Root privileges are required."

  log "Removing keyrs udev rules"
  run_privileged rm -f "${UDEV_RULES_TARGET}"
  run_privileged "${UDEVADM_BIN}" control --reload-rules
  run_privileged "${UDEVADM_BIN}" trigger --subsystem-match=input
  log "udev rules removed."
}

start_cmd() {
  ensure_systemctl_user
  run "${SYSTEMCTL_BIN}" --user start "${SERVICE_NAME}"
}

stop_cmd() {
  ensure_systemctl_user
  run "${SYSTEMCTL_BIN}" --user stop "${SERVICE_NAME}"
}

restart_cmd() {
  ensure_systemctl_user
  run "${SYSTEMCTL_BIN}" --user restart "${SERVICE_NAME}"
}

status_cmd() {
  ensure_systemctl_user
  run "${SYSTEMCTL_BIN}" --user --no-pager --full status "${SERVICE_NAME}"
}

# ============================================================
# Profile Functions
# ============================================================

get_profile_dir() {
  local profile_name="$1"
  # Check source profiles first (dev mode), then installed profiles (runtime mode)
  if [[ -d "${PROFILES_DIR}/${profile_name}" ]]; then
    echo "${PROFILES_DIR}/${profile_name}"
  elif [[ -d "${BUILTIN_PROFILES_DIR}/${profile_name}" ]]; then
    echo "${BUILTIN_PROFILES_DIR}/${profile_name}"
  else
    # Return default for error handling
    echo "${PROFILES_DIR}/${profile_name}"
  fi
}

get_profiles_search_dirs() {
  # Return profile directories in search order
  local dirs=""
  [[ -d "${PROFILES_DIR}" ]] && dirs="${PROFILES_DIR}"
  [[ -d "${BUILTIN_PROFILES_DIR}" ]] && dirs="${dirs} ${BUILTIN_PROFILES_DIR}"
  echo "${dirs}"
}

list_profiles_cmd() {
  local search_dirs
  search_dirs="$(get_profiles_search_dirs)"

  if [[ -z "${search_dirs}" ]]; then
    log_error "No profiles directory found"
    log "Run 'keyrs-service install' to install built-in profiles."
    exit 1
  fi

  log "Available profiles:"
  echo ""

  local found=0
  local seen_names=""
  for dir in ${search_dirs}; do
    for profile_path in "${dir}"/*/profile.toml; do
      if [[ -f "${profile_path}" ]]; then
        local profile_dir
        profile_dir="$(dirname "${profile_path}")"
        local profile_name
        profile_name="$(basename "${profile_dir}")"

        # Skip duplicates (prefer source profiles over installed)
        if [[ "${seen_names}" == *":${profile_name}:"* ]]; then
          continue
        fi
        seen_names="${seen_names}:${profile_name}:"

        found=1
        local name="" description=""
        if command -v grep >/dev/null 2>&1; then
          name="$(grep -E '^name[[:space:]]*=' "${profile_path}" 2>/dev/null | head -1 | sed 's/^name[[:space:]]*=[[:space:]]*//; s/"//g' || true)"
          description="$(grep -E '^description[[:space:]]*=' "${profile_path}" 2>/dev/null | head -1 | sed 's/^description[[:space:]]*=[[:space:]]*//; s/"//g' || true)"
        fi

        printf "  %-20s %s\n" "${profile_name}" "${description:-No description}"
      fi
    done
  done

  if [[ "${found}" -eq 0 ]]; then
    log "No profiles found."
    log "Run 'keyrs-service install' to install built-in profiles."
    exit 1
  fi

  echo ""
  echo "Use 'keyrs-service show-profile <name>' for details."
}

show_profile_cmd() {
  if [[ -z "${SELECTED_PROFILE:-}" ]]; then
    log "Usage: keyrs-service show-profile <profile-name>"
    log "Run 'keyrs-service list-profiles' to see available profiles."
    exit 1
  fi

  local profile_dir
  profile_dir="$(get_profile_dir "${SELECTED_PROFILE}")"
  local profile_toml="${profile_dir}/profile.toml"

  if [[ ! -f "${profile_toml}" ]]; then
    log_error "Profile not found: ${SELECTED_PROFILE}"
    log "Run 'keyrs-service list-profiles' to see available profiles."
    exit 1
  fi

  echo "Profile: ${SELECTED_PROFILE}"
  echo ""
  cat "${profile_toml}"
  echo ""
  echo "Config files:"
  if [[ -d "${profile_dir}/config.d" ]]; then
    for f in "${profile_dir}/config.d"/*.toml; do
      [[ -f "$f" ]] && echo "  - $(basename "$f")"
    done
  else
    echo "  (none)"
  fi
}

download_profile() {
  local url="$1"
  local target_dir="$2"

  log "Downloading profile from ${url}"

  run mkdir -p "${target_dir}"

  local tmp_archive
  tmp_archive="$(mktemp "/tmp/keyrs-profile.XXXXXX.tar.gz")"
  TEMP_FILES+=("${tmp_archive}")

  if command -v curl >/dev/null 2>&1; then
    if ! curl -fsSL "${url}" -o "${tmp_archive}"; then
      log_error "Failed to download profile from ${url}"
      return 1
    fi
  elif command -v wget >/dev/null 2>&1; then
    if ! wget -q "${url}" -O "${tmp_archive}"; then
      log_error "Failed to download profile from ${url}"
      return 1
    fi
  else
    log_error "Neither curl nor wget available for downloading profiles"
    return 1
  fi

  run tar -xzf "${tmp_archive}" -C "${target_dir}" --strip-components=1 2>/dev/null || \
    run tar -xzf "${tmp_archive}" -C "${target_dir}" 2>/dev/null || {
    log_error "Failed to extract profile archive"
    return 1
  }

  rm -f "${tmp_archive}"

  local profile_toml="${target_dir}/profile.toml"
  if [[ ! -f "${profile_toml}" ]]; then
    for subdir in "${target_dir}"/*/; do
      if [[ -f "${subdir}profile.toml" ]]; then
        mv "${subdir}"* "${target_dir}/"
        break
      fi
    done
  fi

  if [[ ! -f "${profile_toml}" ]]; then
    log "Downloaded archive does not contain a valid profile (missing profile.toml)"
    return 1
  fi

  if [[ ! -d "${target_dir}/config.d" ]]; then
    log "Downloaded profile does not contain config.d directory"
    return 1
  fi

  log "Profile downloaded successfully"
  return 0
}

apply_profile() {
  local profile_source_dir="$1"
  local profile_toml="${profile_source_dir}/profile.toml"
  local profile_config_d="${profile_source_dir}/config.d"

  if [[ ! -f "${profile_toml}" ]]; then
    log "Invalid profile: missing profile.toml in ${profile_source_dir}"
    return 1
  fi

  if [[ ! -d "${profile_config_d}" ]]; then
    log "Invalid profile: missing config.d directory in ${profile_source_dir}"
    return 1
  fi

  # Backup existing config
  if [[ -d "${CONFIG_COMPOSE_DIR}" ]]; then
    local backup_dir="${BACKUPS_DIR}/config.d.$(date +%Y%m%d%H%M%S)"
    run mkdir -p "${BACKUPS_DIR}"
    run mv "${CONFIG_COMPOSE_DIR}" "${backup_dir}"
    log "Previous config backed up to: ${backup_dir}"
  fi

  run mkdir -p "${CONFIG_COMPOSE_DIR}"
  run cp -a "${profile_config_d}/." "${CONFIG_COMPOSE_DIR}/"

  log "Applied profile to ${CONFIG_COMPOSE_DIR}"
  return 0
}

# Compose, validate, and restart service (common helper)
validate_and_restart() {
  run "${TARGET_BIN}" --compose-config "${CONFIG_COMPOSE_DIR}" --compose-output "${CONFIG_DIR}/config.toml"
  run "${TARGET_BIN}" --check-config --config "${CONFIG_DIR}/config.toml"
  run "${SYSTEMCTL_BIN}" --user restart "${SERVICE_NAME}"
  run "${SYSTEMCTL_BIN}" --user --no-pager --full status "${SERVICE_NAME}"
}

# Resolve profile directory with error handling
resolve_profile_dir() {
  local profile_name="$1"
  local profile_dir
  profile_dir="$(get_profile_dir "${profile_name}")"

  if [[ ! -f "${profile_dir}/profile.toml" ]]; then
    log_error "Profile not found: ${profile_name}"
    log "Run 'keyrs-service list-profiles' to see available profiles."
    return 1
  fi

  echo "${profile_dir}"
}

profile_set_cmd() {
  ensure_systemctl_user
  resolve_runtime_bin

  # URL mode
  if [[ -n "${PROFILE_URL:-}" ]]; then
    local cache_dir="${PROFILE_CACHE_DIR}/url-$(date +%s)"
    if ! download_profile "${PROFILE_URL}" "${cache_dir}"; then
      exit 1
    fi

    confirm_or_abort \
      "About to set profile from URL:" \
      "  - URL: ${PROFILE_URL}
  - Profile source: ${cache_dir}
  - Target: ${CONFIG_COMPOSE_DIR}
  - Will validate and restart service"

    if ! apply_profile "${cache_dir}"; then
      exit 1
    fi

    validate_and_restart
    log_success "Profile set from URL"
    return
  fi

  # Built-in profile mode
  if [[ -z "${SELECTED_PROFILE:-}" ]]; then
    log "Usage: keyrs-service profile-set <profile-name>"
    log "       keyrs-service profile-set --url <url>"
    log "Run 'keyrs-service list-profiles' to see available profiles."
    exit 1
  fi

  local profile_dir
  if ! profile_dir="$(resolve_profile_dir "${SELECTED_PROFILE}")"; then
    exit 1
  fi

  confirm_or_abort \
    "About to set profile '${SELECTED_PROFILE}':" \
    "  - Profile source: ${profile_dir}
  - Target: ${CONFIG_COMPOSE_DIR}
  - Will validate and restart service"

  if ! apply_profile "${profile_dir}"; then
    exit 1
  fi

  validate_and_restart
  log_success "Profile set: ${SELECTED_PROFILE}"
}

profile_select_cmd() {
  ensure_systemctl_user
  resolve_runtime_bin

  if ! select_profile_interactive; then
    log_error "No profiles available"
    exit 1
  fi

  local profile_dir
  if ! profile_dir="$(resolve_profile_dir "${SELECTED_PROFILE}")"; then
    exit 1
  fi

  if ! apply_profile "${profile_dir}"; then
    exit 1
  fi

  validate_and_restart
  log_success "Profile set: ${SELECTED_PROFILE}"
}

main() {
  parse_args "$@"
  setup_colors
  case "${COMMAND}" in
    # Installation
    install) install_cmd ;;
    uninstall) uninstall_cmd ;;

    # Service
    start) start_cmd ;;
    stop) stop_cmd ;;
    restart) restart_cmd ;;
    status) status_cmd ;;

    # Configuration
    apply-config) apply_config_cmd ;;

    # Profiles
    list-profiles) list_profiles_cmd ;;
    show-profile) show_profile_cmd ;;
    profile-set) profile_set_cmd ;;
    profile-select) profile_select_cmd ;;

    # Advanced
    install-udev) install_udev_cmd ;;
    uninstall-udev) uninstall_udev_cmd ;;

    *)
      log_error "Unknown command: ${COMMAND}"
      usage
      exit 1
      ;;
  esac
}

main "$@"
