-include Makefile.local

DESTDIR ?=
PREFIX ?= /usr/local
BINDIR ?= ${PREFIX}/bin

RM := rm
CARGO := cargo
INSTALL := install

DEBUG := 0
ifeq ($(DEBUG), 0)
	CARGO_OPTIONS := --release --locked
	CARGO_TARGET := release
else
	CARGO_OPTIONS :=
	CARGO_TARGET := debug
endif

.PHONY: all dfrs test clean install uninstall

all: dfrs test

dfrs:
	$(CARGO) build $(CARGO_OPTIONS)

test:
	$(CARGO) test

clean:
	$(RM) -rf target

install:
	$(INSTALL) -Dm 755 target/$(CARGO_TARGET)/dfrs -t $(DESTDIR)$(BINDIR)

uninstall:
	$(RM) -f $(DESTDIR)$(BINDIR)/dfrs
