# 🛡️ BitLocker Control Utility

A fast, Rust-based command-line utility to automate the process of mounting and
unmounting BitLocker-encrypted drives on Linux.

By default, unlocking a BitLocker drive on Linux requires multiple manual
commands (decrypting the volume, creating a loopback file, and mounting the NTFS
filesystem). This tool turns that entire process into a single command:
```bash
bitlocker-control mount
```

### 📖 How it Works (The 2-Step Mount)

Under the hood, this tool uses dislocker. Dislocker requires two mount points:

1.  BitLocker Mount Dir: Where dislocker decrypts the drive and exposes a
    virtual file (usually called dislocker-file).
2.  Final Mount Dir: Where the actual NTFS filesystem is mounted and accessible
    to you.

### 🚀 Quick Start

1. Install System Prerequisites

Ensure you have the required system binaries installed.
On Arch/Manjaro Linux, run:

```bash
paru -S dislocker fuse3 ntfs-3g util-linux --needed
```

or using yay:

```bash
yay -S dislocker fuse3 ntfs-3g util-linux --needed

```

2. Prepare Mount Points

Create the necessary directories (this only needs to be done once).
You need root privileges for this:

```bash
sudo mkdir -p /mnt/bitlocker /mnt/mount
```

3. Install the Utility


```bash
git clone https://github.com/claudiofsr/bitlocker-control.git
cd bitlocker-control
cargo b -r && cargo install --path=.
```

### ⚙️ Configuration

The tool uses a JSON configuration file located at `~/.config/bitlocker-control/config.json`.

Finding your Device Path

To find which device corresponds to your BitLocker drive,
you can use the built-in command:

```bash
bitlocker-control show
```

Alternatively, run lsblk -f.
Look for the partition you want to unlock (e.g., /dev/sda1).

Creating the Config File

Create the config file and populate it with your drive's details.

File: ~/.config/bitlocker-control/config.json

```
┌─[claudio@manjaro] - [~/.config/bitlocker-control] - [seg mai 11, 18:28]
└─[$] <> cat config.json | jq
{
  "device": "/dev/sda1",
  "password": "YourBitLockerPasswordHere",
  "verbose": true,
  "mount_options": "loop,rw,users",
  "bitlocker_mount_dir": "/mnt/bitlocker",
  "final_mount_dir": "/mnt/mount"
}
```

To verify your configuration at any time, run:

```bash
bitlocker-control -c
```

### 🛠️ Usage Examples

Once configured, managing your encrypted drive is simple.

Mount the drive (using config defaults):

```bash
bitlocker-control mount
```

Unmount the drive safely:

```bash
bitlocker-control unmount
```

Unmount and physically eject the drive (for USBs):

```bash
bitlocker-control --eject unmount
```

Override the config to mount a different device:

```bash
bitlocker-control -d /dev/sdb2 -v mount
```

🧰 Command Line Reference

```
┌─[claudio@manjaro] - [~] - [dom mai 10, 19:31]
└─[$] <git:(master)> bitlocker-control help

BitLocker Control Utility
Automates the process of mounting and unmounting BitLocker encrypted drives using dislocker.

Usage: bitlocker-control [OPTIONS] [COMMAND]

Commands:
  mount    Decrypt and mount drive
  unmount  Safely unmount drive
  show     Show block devices (lsblk)
  help     Print this message or the help of the given subcommand(s)

Options:
  -c, --config               Show config file path and content
  -d, --device <DEVICE>      Target block device
  -e, --eject                Eject drive after unmounting
  -p, --password <PASSWORD>  BitLocker password
  -v, --verbose              Show verbose messages
  -q, --quiet                Suppress non-error messages
  -h, --help                 Print help
  -V, --version              Print version

Config file:
  ~/.config/bitlocker-control/config.json

Examples:
  # Standard mount using config defaults
  bitlocker-control mount

  # Mount a specific device with verbose output
  bitlocker-control -d /dev/sda1 -v mount

  # Unmount the device and safely eject it
  bitlocker-control --eject unmount

  # Show current block devices and mount status
  bitlocker-control show

  # Show current configuration content
  bitlocker-control --config

```

### Architecture Style

Architecture Style: Modularized Monolith (Domain-Driven Design Inspired)
Language: Rust (Edition 2024)

This project is separated into a binary target (main.rs) and a library
target (lib.rs) for clean Separation of Concerns (SoC) and testability.
It heavily implements the idiomatic Rust "Fat Library, Thin Binary"
pattern, ensuring safe error handling without abrupt process terminations.


#### DIRECTORY TREE HIERARCHY & RESPONSIBILITIES

```
bitlocker-control/
├── Cargo.toml            # Manifest file containing metadata and dependencies
├── README.txt            # Architectural documentation and design mapping
└── src/
    ├── main.rs           # Entry Point (Thin Wrapper & Error Catcher)
    ├── lib.rs            # Library Root (Core Orchestrator & Exposer)
    ├── cli.rs            # Presentation Layer: Command-line parsing and style
    ├── dependency.rs     # Infrastructure Layer: Arch Linux dependency validations
    ├── error.rs          # Cross-Cutting Layer: App-wide errors (thiserror 2.0)
    ├── logger.rs         # Presentation Layer: Custom ANSI colored output
    ├── config.rs         # Domain Layer: Typed data entities (Config / CommandResult)
    ├── privilege.rs      # Infrastructure Layer: Atomic Unix root privilege escalator
    ├── repository.rs     # Data Access Layer: JSON configuration storage on disk
    ├── service.rs        # Application Layer: BitLocker orchestrator business logic
    └── system.rs         # Infrastructure Layer: Process command executor
```

#### FILE ROLES & ARCHITECTURAL DETAILED DESCRIPTIONS

* main.rs
  - Layer: Application Bootstrap (Wiring)
  - Role: Thin wrapper serving as the system entry point. It delegates the
    entire execution to the library layer (lib.rs) and acts purely as an
    error-catcher, intercepting failures and cleanly terminating the process
    with a graceful exit code.

* lib.rs
  - Layer: Core Library Target
  - Role: Acts as the primary application orchestrator via the `run()` function.
    It manages dependency injection, parses CLI overrides, triggers business
    workflows, and enforces graceful error bubbling using the `Result` pattern.

* cli.rs
  - Layer: Presentation Layer
  - Role: Leverages 'clap' to define structure-driven CLI flags, arguments,
    and colored help layouts, guaranteeing a beautiful user interface.

* dependency.rs
  - Layer: Infrastructure / Gatekeeper
  - Role: Validates that all required OS binaries (dislocker, mount, lsblk, etc.)
    exist in the system PATH. Returns structured errors and provides helpful AUR
    and standard Arch Linux installation command recipes on validation failures.

* error.rs
  - Layer: Cross-Cutting / Domain
  - Role: Centralizes error models via 'thiserror' enum definitions. Decouples system
    failures into structured domain errors, enabling structured rollbacks and
    facilitating the idiomatic Rust `?` operator for elegant error bubbling.

* logger.rs
  - Layer: Presentation Layer
  - Role: Controls colored standard stdout and stderr outputs, enforcing logical
    formatting prefixes ([INFO], [SUCCESS], [WARN], [ERROR]) based on verbosity levels.

* config.rs
  - Layer: Domain Entities
  - Role: Houses strict domain logic structures (Config, CommandResult). These
    data structures are deterministic, self-validating, and hold no OS-side logic.

* privilege.rs
  - Layer: Infrastructure (Unix Security)
  - Role: Identifies the current UID status. If the user is unprivileged, it
    atomically replaces the running thread context with standard 'sudo' delegation,
    propagating an error gracefully if the escalation fails.

* repository.rs
  - Layer: Data Access Layer (Persistence)
  - Role: Ensures that the user state is persisted gracefully. Handles resolving the
    original caller's $HOME directory (bypassing root variables) and performs JSON
    write/read updates and beautiful colored configuration prints on screen.

* service.rs
  - Layer: Application Layer (Business Workflows)
  - Role: Executes domain use-cases. Coordinates directory mapping, dislocker keys,
    loop mounting, clean rollbacks, drive diagnostics, and safe system removals
    returning Results directly to the caller.

* system.rs
  - Layer: Infrastructure (OS IO Wrapper)
  - Role: Executes low-level Unix subprocesses. Emits system activities through
    the logger and returns sanitized output summaries containing code and messages.
