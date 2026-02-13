#!/usr/bin/env bash
set -euo pipefail

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
BIN_DIR="${HOME}/.local/bin"
TARGET_BIN="${BIN_DIR}/keyrs"
TARGET_TUI_BIN="${BIN_DIR}/keyrs-tui"
RUNTIME_CTL="${BIN_DIR}/keyrs-service"

SYSTEMCTL_BIN="${SYSTEMCTL_BIN:-systemctl}"
UDEVADM_BIN="${UDEVADM_BIN:-udevadm}"
DRY_RUN=false
FORCE=false
ASSUME_YES=false
BIN_SOURCE=""
TUI_BIN_SOURCE=""
COMPOSE_SOURCE_DIR=""
RUNTIME_ONLY=false
if [[ "${KEYRS_RUNTIME_ONLY:-0}" == "1" || "${SCRIPT_PATH}" == "${RUNTIME_CTL}" ]]; then
  RUNTIME_ONLY=true
fi

usage() {
  local header
  local examples
  if ${RUNTIME_ONLY}; then
    header="Usage:
  keyrs-service <command> [options]"
    examples="Examples:
  keyrs-service apply-config
  keyrs-service apply-config --source-dir ~/.config/keyrs/config.d --yes
  keyrs-service restart"
  else
    header="Usage:
  scripts/keyrs-service.sh <command> [options]"
    examples="Examples:
  scripts/keyrs-service.sh install
  scripts/keyrs-service.sh install --bin ./target/release/keyrs --force
  scripts/keyrs-service.sh install --yes
  scripts/keyrs-service.sh install-udev
  scripts/keyrs-service.sh restart"
  fi

  cat <<USAGE
${header}

Commands:
  apply-config  Compose+validate config.toml and restart service safely
  start         Start service
  stop          Stop service
  restart       Restart service
  status        Show service status
  install-udev Install keyrs udev rules (root/sudo required)
  uninstall-udev Remove keyrs udev rules (root/sudo required)
USAGE

  if ! ${RUNTIME_ONLY}; then
    cat <<USAGE
  install      Install binary/config and enable/start user service
  uninstall    Stop/disable service and remove service file
USAGE
  fi

  cat <<USAGE
Options:
  --bin <path>     Binary source path (default: ./target/release/keyrs)
  --source-dir <path> Source dir to compose from for apply-config (default: ~/.config/keyrs/config.d)
  --force          Overwrite existing config files during install
  --yes            Skip confirmation prompt
  --dry-run        Print actions without executing system changes
  -h, --help       Show this help

${examples}
USAGE
}

log() {
  printf '[keyrs-service] %s\n' "$*"
}

run() {
  if ${DRY_RUN}; then
    printf '[dry-run] %s\n' "$*"
    return 0
  fi
  "$@"
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
  COMMAND="${1:-}"
  if [[ -z "${COMMAND}" ]]; then
    usage
    exit 1
  fi
  shift || true

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
      --yes)
        ASSUME_YES=true
        shift
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

install_cmd() {
  ensure_systemctl_user
  if ${RUNTIME_ONLY}; then
    log "install is unavailable in runtime mode."
    exit 1
  fi
  resolve_bin_source
  resolve_tui_bin_source

  confirm_or_abort \
    "About to install and activate keyrs service:" \
    "  - Binary source: ${BIN_SOURCE}
  - Install binary: ${TARGET_BIN}
  - Install TUI binary: ${TARGET_TUI_BIN}
  - Config fragments source: ${CONFIG_SOURCE_DIR}
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

  if [[ ! -d "${CONFIG_SOURCE_DIR}" ]]; then
    log "Missing config examples at ${CONFIG_SOURCE_DIR}"
    exit 1
  fi

  if [[ ! -d "${CONFIG_COMPOSE_DIR}" || "${FORCE}" == true ]]; then
    run mkdir -p "${CONFIG_COMPOSE_DIR}"
    run cp -a "${CONFIG_SOURCE_DIR}/." "${CONFIG_COMPOSE_DIR}/"
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

  # Compose and validate final config before service activation.
  run "${TARGET_BIN}" --compose-config "${CONFIG_COMPOSE_DIR}" --compose-output "${CONFIG_DIR}/config.toml"
  run "${TARGET_BIN}" --check-config --config "${CONFIG_DIR}/config.toml"

  write_service_file
  run "${SYSTEMCTL_BIN}" --user daemon-reload
  run "${SYSTEMCTL_BIN}" --user enable --now "${SERVICE_NAME}"
  run "${SYSTEMCTL_BIN}" --user --no-pager --full status "${SERVICE_NAME}"

  log "Install complete"
  log "Recommended once per machine: scripts/keyrs-service.sh install-udev"
}

uninstall_cmd() {
  ensure_systemctl_user
  if ${RUNTIME_ONLY}; then
    log "uninstall is unavailable in runtime mode."
    exit 1
  fi
  confirm_or_abort \
    "About to uninstall keyrs service integration:" \
    "  - Will stop and disable: ${SERVICE_NAME}
  - Will remove service file: ${SERVICE_PATH}
  - Will keep binary/config files in ~/.local/bin and ~/.config/keyrs"

  log "Uninstalling keyrs service"

  run "${SYSTEMCTL_BIN}" --user disable --now "${SERVICE_NAME}" || true
  if [[ -f "${SERVICE_PATH}" ]]; then
    run rm -f "${SERVICE_PATH}"
    log "Removed ${SERVICE_PATH}"
  fi
  run "${SYSTEMCTL_BIN}" --user daemon-reload
  log "Uninstall complete (config and binary kept)"
}

apply_config_cmd() {
  ensure_systemctl_user
  resolve_runtime_bin
  run mkdir -p "${CONFIG_DIR}"

  local source_dir="${COMPOSE_SOURCE_DIR:-${CONFIG_COMPOSE_DIR}}"
  local output_path="${CONFIG_DIR}/config.toml"
  local tmp_path
  tmp_path="$(mktemp "${CONFIG_DIR}/config.toml.new.XXXXXX")"

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

main() {
  parse_args "$@"
  case "${COMMAND}" in
    apply-config) apply_config_cmd ;;
    install) install_cmd ;;
    uninstall) uninstall_cmd ;;
    install-udev) install_udev_cmd ;;
    uninstall-udev) uninstall_udev_cmd ;;
    start) start_cmd ;;
    stop) stop_cmd ;;
    restart) restart_cmd ;;
    status) status_cmd ;;
    *)
      log "Unknown command: ${COMMAND}"
      usage
      exit 1
      ;;
  esac
}

main "$@"
