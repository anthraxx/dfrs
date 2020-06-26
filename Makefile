-include Makefile.local

DESTDIR ?=
PREFIX ?= /usr/local
BINDIR ?= ${PREFIX}/bin
DATAROOTDIR ?= ${PREFIX}/share
MANDIR ?= ${DATAROOTDIR}/man

TARBALLDIR ?= target/release/tarball
TARBALLFORMAT=tar.gz

RM := rm
CARGO := cargo
SCDOC := scdoc
INSTALL := install
GIT := git
GPG := gpg
SED := sed

DEBUG := 0
ifeq ($(DEBUG), 0)
	CARGO_OPTIONS := --release --locked
	CARGO_TARGET := release
else
	CARGO_OPTIONS :=
	CARGO_TARGET := debug
endif

.PHONY: all dfrs test docs man completions clean install uninstall

all: dfrs test docs

dfrs:
	$(CARGO) build $(CARGO_OPTIONS)

test:
	$(CARGO) test $(CARGO_OPTIONS)

lint:
	$(CARGO) fmt -- --check
	$(CARGO) check
	find . -name '*.rs' -exec touch {} +
	$(CARGO) clippy --all -- -D warnings

docs: man completions

man: contrib/man/dfrs.1

contrib/man/%: contrib/man/%.scd
	$(SCDOC) < $^ > $@

completions: dfrs
	target/$(CARGO_TARGET)/dfrs completions bash | $(INSTALL) -Dm 644 /dev/stdin target/completion/bash/dfrs
	target/$(CARGO_TARGET)/dfrs completions zsh | $(INSTALL) -Dm 644 /dev/stdin target/completion/zsh/_dfrs
	target/$(CARGO_TARGET)/dfrs completions fish | $(INSTALL) -Dm 644 /dev/stdin target/completion/fish/dfrs.fish

clean:
	$(RM) -rf target contrib/man/*.1

install: dfrs docs
	$(INSTALL) -Dm 755 target/$(CARGO_TARGET)/dfrs -t $(DESTDIR)$(BINDIR)
	$(INSTALL) -Dm 644 contrib/man/*.1 -t $(DESTDIR)$(MANDIR)/man1
	$(INSTALL) -Dm 644 target/completion/bash/dfrs -t $(DESTDIR)$(DATAROOTDIR)/bash-completion/completions
	$(INSTALL) -Dm 644 target/completion/zsh/_dfrs -t  $(DESTDIR)$(DATAROOTDIR)/zsh/site-functions
	$(INSTALL) -Dm 644 target/completion/fish/dfrs.fish -t  $(DESTDIR)$(DATAROOTDIR)/fish/vendor_completions.d

uninstall:
	$(RM) -f $(DESTDIR)$(BINDIR)/dfrs
	$(RM) -f $(DESTDIR)$(MANDIR)/man1/dfrs.1
	$(RM) -f $(DESTDIR)$(DATAROOTDIR)/bash-completion/completions/dfrs
	$(RM) -f $(DESTDIR)$(DATAROOTDIR)/zsh/site-functions/_dfrs
	$(RM) -f $(DESTDIR)$(DATAROOTDIR)/fish/vendor_completions.d/dfrs.fish

release: all
	$(INSTALL) -d $(TARBALLDIR)
	@read -p 'version> ' TAG && \
		$(SED) "s|version = .*|version = \"$$TAG\"|" -i Cargo.toml && \
		$(CARGO) build --release && \
		$(GIT) commit --gpg-sign --message "version: release $$TAG" Cargo.toml Cargo.lock && \
		$(GIT) tag --sign --message "version: release $$TAG" $$TAG && \
		$(GIT) archive -o $(TARBALLDIR)/dfrs-$$TAG.$(TARBALLFORMAT) --format $(TARBALLFORMAT) --prefix=dfrs-$$TAG/ $$TAG && \
		$(GPG) --detach-sign $(TARBALLDIR)/dfrs-$$TAG.$(TARBALLFORMAT)
