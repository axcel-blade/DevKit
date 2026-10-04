# DevKit build shortcuts. Requires GNU make and a Rust toolchain on PATH.
# `make` prints this list. `make ci` matches the GitHub Actions checks.
#
# Every target first checks that make, cargo, and a C compiler are available
# (bash too, when the goal is `sh`) and installs anything missing.
# If `make` itself is not installed, run scripts/ensure-makefile.bat (Windows)
# or sh scripts/ensure-makefile.sh (macOS / Linux), then run make again.
#
# `make bat` and `make sh` open the plugin menu. Extra words are passed to
# the launcher: `make bat doctor`, `make sh list`.

.DEFAULT_GOAL := help

# When the first goal is bat or sh, later goals are launcher arguments.
ifneq ($(filter $(firstword $(MAKECMDGOALS)),bat sh),)
LAUNCH_ARGS := $(wordlist 2,$(words $(MAKECMDGOALS)),$(MAKECMDGOALS))
else
LAUNCH_ARGS :=
endif

.PHONY: help ensure build release test test-release fmt fmt-check clippy doctor version run bat sh ci clean

help: | ensure
	@echo DevKit
	@echo   make build          Debug build
	@echo   make release        Release build
	@echo   make test           Run tests
	@echo   make test-release   Run tests in release mode
	@echo   make fmt            Format Rust sources
	@echo   make fmt-check      Check formatting
	@echo   make clippy         Lint with warnings denied
	@echo   make doctor         Build and run devkit doctor
	@echo   make version        Print the DevKit version
	@echo   make run            Open the plugin menu via cargo
	@echo   make bat            Run devkit.bat and show the menu
	@echo   make sh             Run devkit.sh and show the menu
	@echo   make bat doctor     Run devkit.bat with arguments
	@echo   make sh list        Run devkit.sh with arguments
	@echo   make ci             fmt-check, clippy, release build, release tests, doctor
	@echo   make clean          Remove build artifacts

ensure:
ifeq ($(OS),Windows_NT)
	@cmd /c scripts\ensure-makefile.bat --from-make $(MAKECMDGOALS)
else
	@sh scripts/ensure-makefile.sh --from-make $(MAKECMDGOALS)
endif

build: | ensure
	cargo build

release: | ensure
	cargo build --release

test: | ensure
	cargo test

test-release: | ensure
	cargo test --release

fmt: | ensure
	cargo fmt --all

fmt-check: | ensure
	cargo fmt --all -- --check

clippy: | ensure
	cargo clippy --all-targets -- -D warnings

ifeq ($(LAUNCH_ARGS),)
doctor: | ensure
	cargo run --release -- doctor
else
doctor:
	@:
endif

version: | ensure
	cargo run --release -- --version

run: | ensure
	cargo run --release

bat: | ensure
ifeq ($(LAUNCH_ARGS),)
	cmd /c devkit.bat
else
	cmd /c devkit.bat $(LAUNCH_ARGS)
endif

sh: | ensure
ifeq ($(LAUNCH_ARGS),)
	bash devkit.sh
else
	bash devkit.sh $(LAUNCH_ARGS)
endif

ci: fmt-check clippy release test-release doctor

clean: | ensure
	cargo clean

# Extra words after `make bat` / `make sh` are not Make targets.
ifneq ($(LAUNCH_ARGS),)
$(foreach goal,$(filter-out doctor,$(LAUNCH_ARGS)),$(eval $(goal):;@:))
endif
