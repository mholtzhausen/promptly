.PHONY: all frontend-deps frontend-build build run test clean install

all: build

frontend-deps:
	npm --prefix frontend ci

frontend-build: frontend-deps
	npm --prefix frontend run build

build: frontend-build
	cargo build --release

run: build
	@if pgrep -x promptly > /dev/null 2>&1; then \
		echo "Stopping existing promptly instance..."; \
		pkill promptly; \
	fi; \
	./target/release/promptly --show

test: frontend-build
	cargo test

clean:
	rm -rf frontend/dist
	cargo clean

install: build
	@RUNNING=$$(pgrep -x promptly > /dev/null 2>&1 && echo 1 || echo 0); \
	sudo cp target/release/promptly /usr/local/bin/; \
	if [ "$$RUNNING" = "1" ]; then \
		echo "Stopping old promptly and restarting..."; \
		pkill promptly; \
		/usr/local/bin/promptly; \
	fi
