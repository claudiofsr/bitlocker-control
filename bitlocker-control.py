#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
BitLocker Control Utility
Architecture: Modularized Monolith (Domain-Driven Design inspired)
Features applied: SOLID principles, Dependency Injection, Strong Typing, Atomic Methods.
"""

import os
import re
import sys
import json
import shutil
import argparse
import subprocess
from dataclasses import dataclass, asdict
from pathlib import Path
from typing import List, Optional

VERSION = "1.0.0"

# ==========================================
# 1. DOMAIN LAYER (Entities & Data Structures)
# ==========================================
# Educational Note: Using dataclasses ensures our configuration has strict types
# and default values, avoiding the pitfalls of unstructured dictionaries.

@dataclass
class AppConfig:
    """Represents the application's configuration state."""
    device: str = "/dev/sda1"
    password: str = "type_your_password_here"
    verbose: bool = True
    mount_options: str = "loop,rw,users"
    bitlocker_mount_dir: str = "/mnt/bitlocker"
    final_mount_dir: str = "/mnt/mount"

@dataclass
class CommandResult:
    """Atomic data structure for standardizing system command outputs."""
    exit_code: int
    output: str
    
    @property
    def is_success(self) -> bool:
        return self.exit_code == 0


# ==========================================
# 2. PRESENTATION LAYER (UI & Logging)
# ==========================================
# Educational Note: This layer handles EVERYTHING related to user output.
# No other class should use print() directly.

class Style:
    """Terminal ANSI escape codes for UI aesthetics."""
    RESET = "\033[0m"
    BOLD = "\033[1m"
    GREEN = "\033[1;32m"
    YELLOW = "\033[1;33m"
    RED = "\033[1;31m"
    BLUE = "\033[1;34m"
    CYAN = "\033[1;36m"
    DIM = "\033[2m"

class Logger:
    """Handles standard output with semantic formatting."""
    def __init__(self, verbose: bool = True):
        self.verbose = verbose

    def info(self, msg: str) -> None:
        if self.verbose:
            print(f"{Style.BLUE}{Style.BOLD}[INFO]{Style.RESET} {msg}")

    def success(self, msg: str) -> None:
        print(f"{Style.GREEN}{Style.BOLD}[SUCCESS]{Style.RESET} {msg}")

    def warning(self, msg: str) -> None:
        print(f"{Style.YELLOW}{Style.BOLD}[WARN]{Style.RESET} {msg}")

    def error(self, msg: str) -> None:
        print(f"{Style.RED}{Style.BOLD}[ERROR]{Style.RESET} {msg}", file=sys.stderr)

    def step(self, msg: str) -> None:
        if self.verbose:
            print(f"{Style.CYAN}{Style.BOLD}➜{Style.RESET} {Style.BOLD}{msg}{Style.RESET}")
            
    def raw(self, msg: str) -> None:
        """Prints unformatted text (useful for command outputs)."""
        print(msg)


# ==========================================
# 3. INFRASTRUCTURE LAYER (System Interactions)
# ==========================================
# Educational Note: This layer isolates the application from the underlying OS.
# If we change the OS, we only rewrite these classes.

class SystemExecutor:
    """Executes external binaries and captures their outputs safely."""
    def __init__(self, logger: Logger):
        self.logger = logger

    def run(self, cmd: List[str]) -> CommandResult:
        self.logger.info(f"Executing: {' '.join(cmd)}")
        try:
            res = subprocess.run(cmd, capture_output=True, text=True, check=False)
            output = res.stderr.strip() if res.stderr else res.stdout.strip()
            return CommandResult(exit_code=res.returncode, output=output)
        except Exception as e:
            return CommandResult(exit_code=1, output=str(e))

class PrivilegeManager:
    """Handles OS privilege elevation atomically."""
    def __init__(self, logger: Logger):
        self.logger = logger

    def ensure_root(self) -> None:
        """Elevates process to root using sudo, replacing the current process."""
        if os.geteuid() == 0:
            return

        self.logger.warning("Root privileges required. Prompting for sudo password...")
        try:
			# 1. Resolve o caminho real do script (essencial para links em ~/bin)
            script_path = os.path.realpath(__file__)
            
            # 2. Prepara os argumentos de forma limpa:
            # [sudo, /usr/bin/python3, /home/claudio/bin/script.py, argumentos...]
            # O sys.argv[1:] pega apenas os comandos/flags, ignorando o nome do script original
            cmd_args = ["sudo", sys.executable, script_path] + sys.argv[1:]
            
            # 3. IMPORTANTE: Limpa os buffers de saída.
            # Se houver texto pendente no buffer do Python, ele pode "sujar" 
            # a entrada do prompt do sudo.
            sys.stdout.flush()
            sys.stderr.flush()
            
            # 4. Substitui o processo atual pelo sudo.
            # O execvp faz o sudo assumir o controle total do TTY (teclado/tela),
            # eliminando interferências do interpretador Python.
            os.execvp("sudo", cmd_args)
        except Exception as e:
            self.logger.error(f"Unexpected error during elevation: {e}")
            sys.exit(1)

class DependencyManager:
    """Checks for required system dependencies and provides Arch Linux install instructions."""
    def __init__(self, logger: Logger):
        self.logger = logger
        # Dictionary format: {"binary_name": "Arch Linux package name"}
        self.required_tools = {
            "dislocker": "dislocker", # Usually in AUR
            "mount": "util-linux",
            "umount": "util-linux",
            "lsblk": "util-linux",
            "eject": "util-linux"
        }

    def check_all(self) -> None:
        """Verifies if all required binaries are in the system PATH."""
        missing = []
        for cmd, pkg in self.required_tools.items():
            if shutil.which(cmd) is None:
                missing.append((cmd, pkg))
                
        if missing:
            self.logger.error("Missing required system dependencies!")
            self._print_install_instructions(missing)
            sys.exit(1)

    def _print_install_instructions(self, missing: List[tuple]) -> None:
        """Generates contextual installation instructions for Arch/Manjaro."""
        print(f"\n{Style.YELLOW}Please install the missing dependencies on your Arch/Manjaro system:{Style.RESET}")
        
        has_aur_deps = any(pkg == "dislocker" for _, pkg in missing)
        standard_pkgs = list(set([pkg for _, pkg in missing if pkg != "dislocker"]))
        
        if standard_pkgs:
            pkgs_str = " ".join(standard_pkgs)
            print(f"  {Style.DIM}# Standard packages via pacman:{Style.RESET}")
            print(f"  {Style.GREEN}sudo pacman -S {pkgs_str}{Style.RESET}")
            
        if has_aur_deps:
            print(f"\n  {Style.DIM}# AUR packages via yay or paru:{Style.RESET}")
            print(f"  {Style.GREEN}yay -S dislocker{Style.RESET}")
            print(f"  {Style.DIM}  -- OR --{Style.RESET}")
            print(f"  {Style.GREEN}paru -S dislocker{Style.RESET}")
        print()


# ==========================================
# 4. DATA ACCESS LAYER (Persistence)
# ==========================================
# Educational Note: Repository pattern abstracts exactly WHERE and HOW 
# the config is saved (JSON, YAML, Database, etc.)

class ConfigRepository:
    """Manages reading and writing AppConfig to disk."""
    def __init__(self, logger: Logger):
        self.logger = logger
        self.config_path = self._get_config_path()

    @staticmethod
    def get_user_home() -> Path:
        """Detects the real user's home to avoid saving config in /root/."""
        sudo_user = os.environ.get("SUDO_USER")
        if sudo_user:
            return Path(os.path.expanduser(f"~{sudo_user}"))
        return Path.home()

    def _get_config_path(self) -> Path:
        """Detects the real user's home to avoid saving config in /root/."""
        base_home = self.get_user_home();
        return base_home / ".config" / "bitlocker-control" / "config.json"

    def load(self) -> AppConfig:
        """Loads configuration from JSON or returns defaults."""
        if not self.config_path.exists():
            return AppConfig()
            
        try:
            with open(self.config_path, "r", encoding="utf-8") as f:
                data = json.load(f)
            # Unpack dict into dataclass, ignoring unknown keys
            valid_keys = AppConfig.__dataclass_fields__.keys()
            filtered_data = {k: v for k, v in data.items() if k in valid_keys}
            return AppConfig(**filtered_data)
        except Exception as e:
            self.logger.error(f"Failed to parse config, using defaults. Error: {e}")
            return AppConfig()

    def save(self, config: AppConfig) -> None:
        """Persists the AppConfig state to JSON."""
        try:
            self.config_path.parent.mkdir(parents=True, exist_ok=True)
            with open(self.config_path, "w", encoding="utf-8") as f:
                json.dump(asdict(config), f, indent=4)
        except IOError as e:
            self.logger.error(f"Could not save config file: {e}")

    def display_pretty_json(self, config: AppConfig) -> None:
        """Prints the configuration with colored JSON keys."""
        print(f"{Style.YELLOW}{Style.BOLD}Configuration Path:{Style.RESET} {Style.BLUE}{self.config_path}{Style.RESET}")
        json_str = json.dumps(asdict(config), indent=4)
        colored_json = re.sub(
            r'(".*?")(?=\s*:)', 
            f"{Style.GREEN}{Style.BOLD}\\1{Style.RESET}", 
            json_str
        )
        print(f"\n{colored_json}\n")


# ==========================================
# 5. APPLICATION LAYER (Business Logic)
# ==========================================
# Educational Note: This class handles the specific use cases.
# It depends on abstractions (injected via constructor) rather than creating them.

class BitLockerService:
    """Coordinates the core business logic of BitLocker volume management."""
    def __init__(self, config: AppConfig, executor: SystemExecutor, logger: Logger):
        self.config = config
        self.executor = executor
        self.logger = logger
        self.b_mnt = Path(self.config.bitlocker_mount_dir)
        self.f_mnt = Path(self.config.final_mount_dir)
        self.dislocker_file = self.b_mnt / "dislocker-file"

    def _ensure_paths_exist(self) -> None:
        """Atomic operation: Prepares required mount directories."""
        for p in [self.b_mnt, self.f_mnt]:
            if not p.exists():
                self.logger.step(f"Creating directory: {p}")
                p.mkdir(parents=True, exist_ok=True)

    def _decrypt_volume(self) -> bool:
        """Atomic operation: Decrypts volume using dislocker."""
        cmd = ["dislocker", "-v", "-V", self.config.device, f"-u{self.config.password}", "--", str(self.b_mnt)]
        result = self.executor.run(cmd)
        if not result.is_success:
            self.logger.error(f"Decryption failed:\n{Style.DIM}{result.output}{Style.RESET}")
            return False
        return True

    def _mount_loop_device(self) -> bool:
        """Atomic operation: Mounts the decrypted file as a loop device."""
        cmd = ["mount", "-o", self.config.mount_options, str(self.dislocker_file), str(self.f_mnt)]
        result = self.executor.run(cmd)
        if not result.is_success:
            self.logger.error(f"Loop mount failed: {result.output}")
            return False
        return True

    def mount(self) -> bool:
        """Use Case 1: Mount a BitLocker drive."""
        self.logger.step(f"Mounting {self.config.device}...")
        self._ensure_paths_exist()
        
        if self._decrypt_volume():
            if self._mount_loop_device():
                self.logger.success(f"Drive successfully available at: {self.f_mnt}")
                return True
            else:
                self.executor.run(["umount", str(self.b_mnt)]) # Rollback
        return False

    def unmount(self, eject_hardware: bool = False) -> bool:
        """Use Case 2: Safely unmount and optionally eject."""
        self.logger.step("Starting unmount cleanup...")
        success_overall = True
        
        for target in [self.f_mnt, self.b_mnt]:
            result = self.executor.run(["umount", str(target)])
            if not result.is_success and "not mounted" not in result.output.lower():
                self.logger.warning(f"Could not unmount {target}: {result.output}")
                success_overall = False
            elif result.is_success:
                self.logger.info(f"Unmounted: {target}")

        self.logger.success("Cleanup complete.")

        if eject_hardware:
            self.logger.step(f"Ejecting {self.config.device}...")
            result = self.executor.run(["eject", self.config.device])
            if result.is_success:
                self.logger.success("Hardware ejected safely.")
            else:
                self.logger.error(f"Eject failed: {result.output}")
                success_overall = False
                
        return success_overall

    def show_status(self) -> None:
        """Use Case 3: Display block device layout."""
        self.logger.step("Retrieving block device status...")
        result = self.executor.run(["lsblk", "-f"])
        if result.is_success:
            self.logger.raw(f"\n{Style.DIM}{result.output}{Style.RESET}\n")
        else:
            self.logger.error(f"Could not retrieve lsblk status: {result.output}")


# ==========================================
# 6. CLI INTERFACE (Argument Parsing)
# ==========================================

class CLIParser:
    """Encapsulates argument parsing logic."""
    @staticmethod
    def parse(config_repo: ConfigRepository, current_config: AppConfig) -> argparse.Namespace:
        class CustomHelpFormatter(argparse.ArgumentParser):
            def print_help(self):
                print(f"""{Style.BOLD}BitLocker Control Utility v{VERSION}{Style.RESET}

Automates the process of mounting and unmounting BitLocker encrypted drives using dislocker.

{Style.YELLOW}{Style.BOLD}Usage:{Style.RESET} bitlocker-control [OPTIONS] [COMMAND]

{Style.YELLOW}{Style.BOLD}Options:{Style.RESET}
  {Style.GREEN}-c, --config{Style.RESET}          Show config file path and content
  {Style.GREEN}-d, --device <DEV>{Style.RESET}    Target device [default: {current_config.device}]
  {Style.GREEN}-e, --eject{Style.RESET}           Eject drive after unmounting
  {Style.GREEN}-p, --password <PASS>{Style.RESET} BitLocker password
  {Style.GREEN}-v, --verbose{Style.RESET}         Show verbose messages
  {Style.GREEN}-q, --quiet{Style.RESET}           Suppress non-error messages
  {Style.GREEN}-V, --version{Style.RESET}         Show version info

{Style.YELLOW}{Style.BOLD}Commands:{Style.RESET}
  {Style.GREEN}mount{Style.RESET}                 Decrypt and mount drive
  {Style.GREEN}unmount{Style.RESET}               Safely unmount drive
  {Style.GREEN}show{Style.RESET}                  Show block devices (lsblk)

{Style.YELLOW}{Style.BOLD}Config file:{Style.RESET}
  {Style.BLUE}{config_repo.config_path}{Style.RESET}

{Style.YELLOW}{Style.BOLD}Examples:{Style.RESET}
  {Style.DIM}# Standard mount using config defaults{Style.RESET}
  {Style.GREEN}bitlocker-control mount{Style.RESET}

  {Style.DIM}# Mount a specific device with verbose output{Style.RESET}
  {Style.GREEN}bitlocker-control -d /dev/sda1 -v mount{Style.RESET}

  {Style.DIM}# Unmount the device and safely eject it{Style.RESET}
  {Style.GREEN}bitlocker-control --eject unmount{Style.RESET}

  {Style.DIM}# Show current block devices and mount status{Style.RESET}
  {Style.GREEN}bitlocker-control show{Style.RESET}

  {Style.DIM}# Show current configuration content{Style.RESET}
  {Style.GREEN}bitlocker-control --config{Style.RESET}
""")

            def error(self, message):
                print(f"\n{Style.RED}{Style.BOLD}[ERROR]{Style.RESET} {message}")
                self.print_help()
                sys.exit(2)

        parser = CustomHelpFormatter(add_help=False)
        parser.add_argument("-c", "--config", action="store_true")
        parser.add_argument("-d", "--device")
        parser.add_argument("-e", "--eject", action="store_true")
        parser.add_argument("-p", "--password")
        parser.add_argument("-v", "--verbose", action="store_true")
        parser.add_argument("-q", "--quiet", action="store_true")
        parser.add_argument("-V", "--version", action="store_true")
        parser.add_argument("-h", "--help", action="store_true")
        parser.add_argument("command", nargs="?", choices=["mount", "unmount", "show"])

        args = parser.parse_args()

        if args.help or len(sys.argv) == 1:
            parser.print_help()
            sys.exit(0)

        if args.version:
            print(f"bitlocker-control v{VERSION}")
            sys.exit(0)

        if not args.command and not args.config:
            parser.error("No command provided.")

        return args


# ==========================================
# 7. DEPENDENCY INJECTION & EXECUTION (Main)
# ==========================================

def main():
    """Main execution flow: wires dependencies together and executes use cases."""
    
    # 1. Initialize core system utilities (Bootstrap)
    temp_logger = Logger(verbose=False)
    config_repo = ConfigRepository(temp_logger)
    
    # 2. Load existing state
    config = config_repo.load()

    # 3. Parse user inputs
    args = CLIParser.parse(config_repo, config)

    # 4. Handle immediate actions (--config)
    if args.config:
        config_repo.display_pretty_json(config)
        sys.exit(0)

    # 5. Apply CLI overrides to Domain Model
    if args.device: config.device = args.device
    if args.password: config.password = args.password
    if args.verbose: config.verbose = True
    if args.quiet: config.verbose = False

    # 6. Re-initialize services with updated config
    logger = Logger(verbose=config.verbose)
    executor = SystemExecutor(logger)
    privilege_manager = PrivilegeManager(logger)
    dependency_manager = DependencyManager(logger)
    bl_service = BitLockerService(config, executor, logger)

    # 7. Pre-flight checks
    dependency_manager.check_all()

    if args.command in ["mount", "unmount"]:
        privilege_manager.ensure_root()

    # 8. Business Logic Execution
    try:
        success = False
        if args.command == "mount":
            success = bl_service.mount()
        elif args.command == "unmount":
            success = bl_service.unmount(eject_hardware=args.eject)
        elif args.command == "show":
            bl_service.show_status()
            sys.exit(0) 

        # 9. Persistence (Save only on success)
        if success:
            config_repo.save(config)
            logger.info("Configuration updated with last successful parameters.")
        else:
            sys.exit(1)

    except KeyboardInterrupt:
        print()
        logger.warning("Process interrupted by user.")
        sys.exit(130)

if __name__ == "__main__":
    main()
