.PHONY: all build install

all: build install

build:
	@echo "Building the CLI..."
	@cargo build --release
	@echo "Build complete."

install: build
	@echo "Copying the binary to the destination..."
	@mkdir -p ~/emacs.d/valis/
	@cp ./target/release/valis_cli ~/emacs.d/valis/valis
	@echo "Copy complete."
