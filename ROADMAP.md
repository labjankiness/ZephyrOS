# ZephyrOS Roadmap

This roadmap is organized into major phases.

## Phase 1 — VM-stable with Secure Boot [Complete]

Goal: a reproducible ZephyrOS ISO that boots reliably in VMs with Secure Boot,
showcasing the core Wayfire desktop.

Milestones:

- ISO build:
  - Arch-based `archiso` profile under `build/`.
  - Single-command build via `build/build-iso.sh`.
- Boot and Secure Boot:
  - UEFI-only boot via shim → GRUB2 → signed kernel.
  - Development MOK workflow documented and scripted under `secureboot/`.
  - `vm/test-secureboot.sh` validates Secure Boot state inside the guest.
- Desktop:
  - Wayfire compositor with a Windows 11-inspired shell (centered taskbar,
    frosted launcher, rounded corners).
  - Minimal app set (Thunar, Kitty, Wofi, browser/editor TBD).
- Tooling:
  - QEMU/KVM + OVMF launch script (`vm/run-vm.sh`) and defaults (`vm.env`).
  - Theming/dotfiles pipeline via `theme/apply-theme.sh`.

## Phase 1.5 — Edition System with AI Integration [Complete]

Goal: multiple ZephyrOS editions, each bundled with an AI model via Ollama.

Milestones:

- Edition system:
  - Base + edition overlay architecture under `editions/`.
  - `build/build-edition.sh` merges packages and builds per-edition ISOs.
  - 5 editions: Core, Scholar, Dev, SOC, Lite.
- AI integration:
  - Ollama bundled in base packages.
  - First-boot systemd service auto-pulls the edition's AI model.
  - `zephyros-ai` wrapper with edition-specific system prompts.
- Edition-specific packages and MOTDs for each edition.

## Phase 2 — Full Installer Experience [Complete]

Goal: download ISO → boot VM/bare metal → install → reboot → use ZephyrOS.

Milestones:

- TUI installer (`zephyros-install`):
  - Dialog-based guided installation with disk selection, partitioning,
    user account creation, timezone, and bootloader setup.
  - GPT partitioning with EFI (512MB) + root (ext4).
  - GRUB bootloader installation for UEFI.
  - Copies edition config, AI wrapper, first-boot service, and dotfiles.
  - Auto-login + Wayfire auto-start on installed system.
- Live session experience:
  - Welcome dialog on boot with Install / Try / Terminal options.
  - "Install ZephyrOS" desktop shortcut and application entry.
  - Package list bundled in ISO for offline-capable installation.
- Merged package list included in ISO so pacstrap can install the full
  edition package set during installation.

## Phase 3 — Bare metal testing + hardware compatibility [Scaffolded]

Goal: ZephyrOS runs well on a range of real hardware.

Milestones:

- Hardware coverage:
  - GPUs: Intel, AMD, and at least one Nvidia path documented.
  - Wi-Fi/Bluetooth: verify common chipsets, document any non-free firmware.
- Power and performance:
  - Suspend/resume and basic power management on laptops.
  - Boot-time analysis with `systemd-analyze` and service tuning.
- Telemetry stance:
  - Confirm that no telemetry or tracking packages are shipped by default.

What landed (scaffolding):

- Driver/firmware coverage in `editions/base/packages.x86_64`: `sof-firmware`,
  `intel-ucode`/`amd-ucode` microcode, `vulkan-nouveau`, VA-API/VDPAU
  stacks, `iw`/`wireless-regdb`/`wpa_supplicant`, `bluez`/`bluez-utils`.
- Power and thermal: `tlp`, `tlp-rdw`, `thermald`, `upower`, `acpid`,
  `pm-utils` plus a baseline drop-in at `/etc/tlp.d/00-zephyros.conf`.
  The installer enables `tlp`, `thermald`, and `bluetooth` services on the
  target system.
- Diagnostics shipped on every edition:
  - `zephyros-hwreport` — CPU/GPU/Wi-Fi/BT/audio/firmware snapshot.
  - `zephyros-bootreport` — `systemd-analyze` wrap with slow-unit flags.
  - `zephyros-telemetry-audit` — static scan against a deny-list of known
    telemetry packages/units/hosts.

Still requires bare-metal work:

- Running the ISOs on real Intel/AMD/Nvidia laptops and desktops.
- Filing compatibility reports (generated via `zephyros-hwreport`) per
  tested machine.
- Tuning boot services based on real-world `zephyros-bootreport` output.
- Verifying suspend/resume across tested chipsets.

### VM validation log

No bare-metal hardware available, so pre-bare-metal validation is being done
by building an edition ISO and booting it in QEMU/KVM+OVMF instead — this
catches build/installer bugs before ever touching real hardware, but does not
satisfy the bare-metal milestones above.

- **2026-09-09**: Building the Core edition ISO inside a privileged Arch
  Linux container (podman) on an Ubuntu 24.04 / WSL2 host, to then boot with
  `vm/run-vm.sh` (qemu-system-x86_64 + OVMF, `/dev/kvm` available). In
  progress as of this note — update this log with the outcome (boot success/
  failure, Secure Boot state, installer run) once it completes.

## Phase 4 — Public ISO release [Scaffolded]

Goal: polished ZephyrOS ISO with public documentation and a clear Secure Boot
story.

Milestones:

- Release process:
  - Versioned ISOs with checksums and signatures.
  - Public changelog and upgrade guidance.
- Secure Boot hardening:
  - Shim submitted and, if accepted, signed by the Microsoft UEFI CA.
  - Finalized key management and revocation policy.
- Documentation:
  - User guide for installation, updates, and basic customization.
  - Developer guide for rebuilding the ISO and contributing changes.
- Presentation:
  - Website or landing page describing ZephyrOS goals and download links.

What landed (scaffolding):

- Tag-triggered release workflow in `.github/workflows/release.yml`:
  builds all 5 editions, generates SHA-256 and SHA-512 checksums, GPG-signs
  ISOs and checksums (conditional on `GPG_PRIVATE_KEY`/`GPG_KEY_ID`/
  `GPG_PASSPHRASE` secrets), aggregates `SHA256SUMS`/`SHA512SUMS`, and
  auto-generates release notes with the changelog since the previous tag.
- Key policy in [KEYS.md](KEYS.md): separation of Release/Shim/Kernel/Module
  keys, rotation schedule, revocation flow, end-user verification recipe.
- [USER-GUIDE.md](USER-GUIDE.md) and [DEVELOPER-GUIDE.md](DEVELOPER-GUIDE.md)
  cover install, verification, upgrades, and the build/contribution loop.

Still requires external action:

- Preparing and submitting the ZephyrOS shim to the Microsoft UEFI CA.
- Generating the production key set per KEYS.md on offline hardware.
- Populating the release-signing secrets in the repository.
- Cutting the first public tag (`v0.x.0`) to exercise the workflow end-to-end.
