#!/usr/bin/env bash
# bootstrap-linux.sh -- Install the Rust stable toolchain for FileForge Workbench
#
# Usage:
#   bash bootstrap/bootstrap-linux.sh
#   bash bootstrap/bootstrap-linux.sh --toolchain beta
#
# Installs into ~/.tools/rust (no sudo required).
# Safe to run more than once: skips steps already complete.
#
# After this script succeeds, run:
#   source ~/.profile
#   cargo build
# from the repository root, or see tools/powershell/ffwb_make.ps1 for the
# full build-test-run workflow (PowerShell 7 required on Linux).

set -euo pipefail

# === Configuration ===========================================================

TOOLCHAIN="${1:-stable}"
for arg in "$@"; do
    case "$arg" in
        --toolchain=*) TOOLCHAIN="${arg#*=}" ;;
        --toolchain)   shift; TOOLCHAIN="${1:-stable}" ;;
    esac
done

TOOLS_ROOT="${HOME}/.tools"
RUST_DIR="${TOOLS_ROOT}/rust"
export CARGO_HOME="${RUST_DIR}/cargo"
export RUSTUP_HOME="${RUST_DIR}/rustup"
CARGO_BIN="${CARGO_HOME}/bin"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOGS_DIR="${SCRIPT_DIR}/logs"
TIMESTAMP="$(date '+%Y%m%d-%H%M%S')"
LOG_FILE="${LOGS_DIR}/bootstrap-linux-${TIMESTAMP}.log"

# === Logging =================================================================

mkdir -p "${LOGS_DIR}"

log() {
    local level="${2:-INFO}"
    local line="[$(date '+%H:%M:%S')] [${level}] ${1}"
    echo "${line}"
    echo "${line}" >> "${LOG_FILE}"
}

# === Idempotency check =======================================================

log "=== FileForge Workbench -- Rust Bootstrap (Linux) ==="
log "CARGO_HOME:  ${CARGO_HOME}"
log "RUSTUP_HOME: ${RUSTUP_HOME}"
log "Toolchain:   ${TOOLCHAIN}"

if [ -x "${CARGO_BIN}/rustc" ]; then
    log "Rust already installed -- skipping download and install."
else

    # === Download rustup installer ============================================

    RUSTUP_SH="$(mktemp /tmp/rustup-init-XXXXXX.sh)"
    trap 'rm -f "${RUSTUP_SH}"' EXIT

    log "Downloading rustup installer..."
    if command -v curl >/dev/null 2>&1; then
        curl -sSf https://sh.rustup.rs -o "${RUSTUP_SH}"
    elif command -v wget >/dev/null 2>&1; then
        log "curl not found, falling back to wget." "WARN"
        wget -qO "${RUSTUP_SH}" https://sh.rustup.rs
    else
        log "ERROR: neither curl nor wget is available." "ERROR"
        exit 1
    fi

    chmod +x "${RUSTUP_SH}"

    # === Install Rust =========================================================

    mkdir -p "${CARGO_HOME}" "${RUSTUP_HOME}"

    log "Installing Rust ${TOOLCHAIN} ..."
    sh "${RUSTUP_SH}" \
        -y \
        --no-modify-path \
        --default-toolchain "${TOOLCHAIN}" \
        --profile default

    if [ ! -x "${CARGO_BIN}/rustc" ]; then
        log "ERROR: rustc not found after installation." "ERROR"
        exit 1
    fi

    log "Rust installed successfully."
fi

# === Update PATH in shell profiles ===========================================

PATH_LINE="export PATH=\"${CARGO_BIN}:\$PATH\""

for profile in "${HOME}/.profile" "${HOME}/.bashrc"; do
    if [ -f "${profile}" ]; then
        if grep -qF "${CARGO_BIN}" "${profile}" 2>/dev/null; then
            log "${profile}: PATH already contains cargo/bin -- no change."
        else
            printf '\n# Added by FileForge Workbench bootstrap\n%s\n' "${PATH_LINE}" >> "${profile}"
            log "Appended PATH export to ${profile}."
        fi
    else
        printf '# Added by FileForge Workbench bootstrap\n%s\n' "${PATH_LINE}" > "${profile}"
        log "Created ${profile} with PATH export."
    fi
done

# === Verify ==================================================================

export PATH="${CARGO_BIN}:${PATH}"

if command -v rustc >/dev/null 2>&1; then
    log "rustc:  $(rustc --version)"
fi
if command -v cargo >/dev/null 2>&1; then
    log "cargo:  $(cargo --version)"
fi

# === Next steps ==============================================================

log "=== Bootstrap complete ==="

cat <<EOF

========================================
  RUST BOOTSTRAP COMPLETE
========================================

  CARGO_HOME:  ${CARGO_HOME}
  RUSTUP_HOME: ${RUSTUP_HOME}
  Log file:    ${LOG_FILE}

  Next steps:
    1. source ~/.profile   (or open a new terminal)
    2. cd to the repository root
    3. cargo build
    4. cargo test
    5. ./target/debug/ffwb

EOF
