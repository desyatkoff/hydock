BINDIR = /usr/bin
TARGET = hydock

all: clean build install config

build:
	cargo build --release --verbose

clean:
	cargo clean --verbose
	rm -fv $(BINDIR)/$(TARGET)

config:
	mkdir -p $(HOME)/.config/$(TARGET)/
	cp -fv ./assets/config.toml $(HOME)/.config/$(TARGET)/config.toml
	cp -fv ./assets/style.css $(HOME)/.config/$(TARGET)/style.css

install:
	install -D -m755 -v ./target/release/$(TARGET) $(BINDIR)/$(TARGET)

uninstall:
	rm -fv $(BINDIR)/$(TARGET)
	rm -frv $(HOME)/.config/$(TARGET)/

.PHONY: all build clean install uninstall
