#!/usr/bin/env sh
#
# run-vm.sh - Launch ZephyrOS ISO in QEMU/KVM with OVMF Secure Boot

set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
ROOT_DIR=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)

# Load defaults
if [ -f "$SCRIPT_DIR/vm.env" ]; then
    # shellcheck disable=SC1090
    . "$SCRIPT_DIR/vm.env"
fi

QEMU_BIN=${QEMU_BIN:-qemu-system-x86_64}

log() {
    printf '%s\n' "$*" >&2
}

abort() {
    log "ERROR: $*"
    exit 1
}

require_cmd() {
    command -v "$1" >/dev/null 2>&1 || abort "Required command '$1' not found. Install it and retry."
}

latest_iso() {
    ISO_DIR="$ROOT_DIR/build/out"
    [ -d "$ISO_DIR" ] || return 1
    ls -1t "$ISO_DIR"/zephyros*.iso 2>/dev/null | head -n 1
}

ensure_disk() {
    IMAGE_PATH="$1"
    SIZE_GB="$2"

    if [ -f "$IMAGE_PATH" ]; then
        return 0
    fi

    log "Creating VM disk image: $IMAGE_PATH (${SIZE_GB}G)"
    qemu-img create -f qcow2 "$IMAGE_PATH" "${SIZE_GB}G" >/dev/null
}

main() {
    require_cmd "$QEMU_BIN"
    require_cmd qemu-img

    ISO=${ISO_PATH:-$(latest_iso || true)}
    if [ -z "${ISO:-}" ]; then
        abort "No ZephyrOS ISO found in build/out/. Build the ISO first with: (cd build && ./build-iso.sh)"
    fi

    RAM_MB=${VM_RAM_MB:-4096}
    CPUS=${VM_CPUS:-2}

    DISK_IMAGE_NAME=${VM_DISK_IMAGE:-zephyros-vm.qcow2}
    DISK_IMAGE="$SCRIPT_DIR/$DISK_IMAGE_NAME"
    DISK_GB=${VM_DISK_GB:-32}

    ensure_disk "$DISK_IMAGE" "$DISK_GB"

    UEFI_CODE=${VM_UEFI_CODE:-}
    UEFI_VARS=${VM_UEFI_VARS:-}

    [ -n "$UEFI_CODE" ] && [ -f "$UEFI_CODE" ] || abort "UEFI code image not found (VM_UEFI_CODE=$UEFI_CODE)."

    if [ -n "$UEFI_VARS" ] && [ ! -f "$UEFI_VARS" ]; then
        log "UEFI vars image not found at $UEFI_VARS, attempting to copy from template..."
        cp "$UEFI_CODE" "$UEFI_VARS"
    fi

    [ -n "$UEFI_VARS" ] && [ -f "$UEFI_VARS" ] || abort "UEFI vars image not found (VM_UEFI_VARS=$UEFI_VARS)."

    DISPLAY_BACKEND=${VM_DISPLAY:-gtk}

    KVM_FLAGS=""
    MACHINE_OPTS="-machine q35"
    if [ -e /dev/kvm ]; then
        KVM_FLAGS="-enable-kvm"
        MACHINE_OPTS="-machine q35,accel=kvm"
    else
        log "Warning: /dev/kvm not present, running without KVM acceleration."
    fi

    # GL/virgl acceleration requires a working host DRI device, and OVMF's
    # QemuVideoDxe driver (used by most distro OVMF builds) only speaks the
    # classic "VGA" device, not virtio-gpu/virtio-vga. Some hosts (e.g.
    # WSLg) also advertise a display but have no functional GL passthrough,
    # so allow disabling GL via VM_GL=off, falling back to plain VGA.
    GL_MODE=${VM_GL:-on}
    if [ "$GL_MODE" = "on" ]; then
        GPU_DEVICE="virtio-vga-gl"
        GL_SUFFIX=",gl=on"
    else
        GPU_DEVICE="VGA"
        GL_SUFFIX=""
    fi

    DISPLAY_OPTS=""
    if [ -n "${VM_DISPLAY:-}" ]; then
        DISPLAY_OPTS="-display $DISPLAY_BACKEND$GL_SUFFIX"
    else
        if [ -z "${DISPLAY:-}" ] && [ -z "${WAYLAND_DISPLAY:-}" ]; then
            log "No DISPLAY or WAYLAND_DISPLAY detected; running headless with VNC on :0."
            DISPLAY_OPTS="-display none -vnc :0"
        else
            DISPLAY_OPTS="-display $DISPLAY_BACKEND$GL_SUFFIX"
        fi
    fi

    log "Launching ZephyrOS VM..."
    log "  ISO     : $ISO"
    log "  RAM     : ${RAM_MB} MB"
    log "  vCPUs   : $CPUS"
    log "  Disk    : $DISK_IMAGE"
    log "  UEFI    : code=$UEFI_CODE vars=$UEFI_VARS"

    exec "$QEMU_BIN" \
        $KVM_FLAGS \
        -m "$RAM_MB" \
        -smp "$CPUS" \
        $MACHINE_OPTS \
        -device virtio-scsi-pci,id=scsi0 \
        -device virtio-net-pci,netdev=net0 \
        -netdev user,id=net0 \
        -drive if=none,file="$DISK_IMAGE",id=drive0,format=qcow2 \
        -device scsi-hd,drive=drive0 \
        -cdrom "$ISO" \
        -boot order=d \
        $DISPLAY_OPTS \
        -device $GPU_DEVICE \
        -drive if=pflash,format=raw,readonly=on,file="$UEFI_CODE" \
        -drive if=pflash,format=raw,file="$UEFI_VARS" \
        ${VM_EXTRA_ARGS:-}
}

main "$@"

