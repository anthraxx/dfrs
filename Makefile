-include Makefile.local

DESTDIR ?=
PREFIX ?= /usr/local
BINDIR ?= ${PREFIX}/bin
DATAROOTDIR ?= ${PREFIX}/share
MANDIR ?= ${DATAROOTDIR}/man

RM := rm
CARGO := cargo
SCDOC := scdoc
INSTALL := install

DEBUG := 0
ifeq ($(DEBUG), 0)
	CARGO_OPTIONS := --release --locked
	CARGO_TARGET := release
else
	CARGO_OPTIONS :=
	CARGO_TARGET := debug
endif

.PHONY: all dfrs test docs man clean install uninstall

all: dfrs test docs

dfrs:
	$(CARGO) build $(CARGO_OPTIONS)

test:
	$(CARGO) test

docs man: contrib/man/dfrs.1

contrib/man/%: contrib/man/%.scd
	$(SCDOC) < $^ > $@

clean:
	$(RM) -rf target contrib/man/*.1

install:
	$(INSTALL) -Dm 755 target/$(CARGO_TARGET)/dfrs -t $(DESTDIR)$(BINDIR)
	$(INSTALL) -Dm 644 contrib/man/*.1 -t $(DESTDIR)$(MANDIR)/man1

uninstall:
	$(RM) -f $(DESTDIR)$(BINDIR)/dfrs
	$(RM) -f $(DESTDIR)$(MANDIR)/man1/dfrs.1
