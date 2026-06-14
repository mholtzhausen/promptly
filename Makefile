.PHONY: all build run test clean install

all: build

libxdo:
	mkdir -p lib
	if [ ! -f lib/libxdo.so ]; then \
		if [ -f /usr/lib/x86_64-linux-gnu/libxdo.so.3 ]; then \
			ln -sf /usr/lib/x86_64-linux-gnu/libxdo.so.3 lib/libxdo.so; \
		elif [ -f /usr/lib/libxdo.so.3 ]; then \
			ln -sf /usr/lib/libxdo.so.3 lib/libxdo.so; \
		else \
			echo "Error: libxdo.so.3 not found on system." && exit 1; \
		fi \
	fi

build: libxdo
	RUSTFLAGS="-L ./lib" cargo build --release

run: build
	./target/release/promptly

test: libxdo
	RUSTFLAGS="-L ./lib" cargo test

clean:
	rm -rf lib
	cargo clean

install: build
	@RUNNING=$$(pgrep -x promptly > /dev/null 2>&1 && echo 1 || echo 0); \
	sudo cp target/release/promptly /usr/local/bin/; \
	if [ "$$RUNNING" = "1" ]; then \
		echo "Stopping old promptly and restarting..."; \
		pkill promptly; \
		/usr/local/bin/promptly; \
	fi
