.PHONY: all frontend-deps frontend-build build run test lint clean install install-user uninstall

all: build

frontend-deps:
	npm --prefix frontend ci

frontend-build: frontend-deps
	npm --prefix frontend run build

build: frontend-build
	cargo build --release

run: build
	@PROMPTLY_FOREGROUND=1 ./target/release/promptly --show

test: frontend-build
	cargo test
	npm --prefix frontend test

lint:
	cargo fmt --all -- --check
	cargo clippy --all-targets -- -D warnings
	npm --prefix frontend run typecheck

clean:
	rm -rf frontend/dist
	cargo clean

# System-wide install (requires sudo)
install: build
	install -d /usr/local/bin
	install -m 755 target/release/promptly /usr/local/bin/promptly
	install -d /usr/share/applications
	install -m 644 packaging/promptly.desktop /usr/share/applications/promptly.desktop

# User-local install (~/.local)
install-user: build
	install -d $(HOME)/.local/bin $(HOME)/.local/share/applications $(HOME)/.config/autostart $(HOME)/.config/systemd/user
	install -m 755 target/release/promptly $(HOME)/.local/bin/promptly
	install -m 644 packaging/promptly.desktop $(HOME)/.local/share/applications/promptly.desktop
	sed 's|^Exec=promptly|Exec=$(HOME)/.local/bin/promptly|' packaging/promptly.desktop > $(HOME)/.config/autostart/promptly.desktop
	install -m 644 packaging/promptly.service $(HOME)/.config/systemd/user/promptly.service
	@echo "Installed to $(HOME)/.local/bin/promptly"
	@echo "Enable autostart: systemctl --user enable --now promptly.service"

uninstall:
	rm -f $(HOME)/.local/bin/promptly
	rm -f $(HOME)/.local/share/applications/promptly.desktop
	rm -f $(HOME)/.config/autostart/promptly.desktop
	rm -f $(HOME)/.config/systemd/user/promptly.service
	@echo "Removed user-local Promptly install (config and data kept in ~/.config/promptly)"
