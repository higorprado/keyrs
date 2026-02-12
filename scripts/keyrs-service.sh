#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

SERVICE_NAME="keyrs.service"
SERVICE_DIR="${HOME}/.config/systemd/user"
SERVICE_PATH="${SERVICE_DIR}/${SERVICE_NAME}"
CONFIG_DIR="${HOME}/.config/keyrs"
CONFIG_SOURCE_DIR="${REPO_ROOT}/config.d.example"
CONFIG_COMPOSE_DIR="${CONFIG_DIR}/config.d"
BIN_DIR="${HOME}/.local/bin"
TARGET_BIN="${BIN_DIR}/keyrs"

SYSTEMCTL_BIN="${SYSTEMCTL_BIN:-systemctl}"
DRY_RUN=false
FORCE=false
BIN_SOURCE=""

usage() {
  cat <<USAGE
Usage:
  scripts/keyrs-service.sh <command> [options]

Commands:
  install      Install binary/config and enable/start user service
  uninstall    Stop/disable service and remove service file
  start        Start service
  stop         Stop service
  restart      Restart service
  status       Show service status

Options:
  --bin <path>     Binary source path (default: ./target/release/keyrs)
  --force          Overwrite existing config files during install
  --dry-run        Print actions without executing system changes
  -h, --help       Show this help

Examples:
  scripts/keyrs-service.sh install
  scripts/keyrs-service.sh install --bin ./target/release/keyrs --force
  scripts/keyrs-service.sh restart
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
  resolve_bin_source

  log "Installing keyrs service"
  run mkdir -p "${BIN_DIR}" "${CONFIG_DIR}" "${SERVICE_DIR}"
  run install -m 755 "${BIN_SOURCE}" "${TARGET_BIN}"

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

  # Compose and validate final config before service activation.
  run "${TARGET_BIN}" --compose-config "${CONFIG_COMPOSE_DIR}" --compose-output "${CONFIG_DIR}/config.toml"
  run "${TARGET_BIN}" --check-config --config "${CONFIG_DIR}/config.toml"

  write_service_file
  run "${SYSTEMCTL_BIN}" --user daemon-reload
  run "${SYSTEMCTL_BIN}" --user enable --now "${SERVICE_NAME}"
  run "${SYSTEMCTL_BIN}" --user --no-pager --full status "${SERVICE_NAME}"

  log "Install complete"
}

uninstall_cmd() {
  ensure_systemctl_user
  log "Uninstalling keyrs service"

  run "${SYSTEMCTL_BIN}" --user disable --now "${SERVICE_NAME}" || true
  if [[ -f "${SERVICE_PATH}" ]]; then
    run rm -f "${SERVICE_PATH}"
    log "Removed ${SERVICE_PATH}"
  fi
  run "${SYSTEMCTL_BIN}" --user daemon-reload
  log "Uninstall complete (config and binary kept)"
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
    install) install_cmd ;;
    uninstall) uninstall_cmd ;;
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
