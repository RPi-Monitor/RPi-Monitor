# Makefile for the RPi-Monitor documentation: HTML book (mdBook) and man pages (pandoc)

BUILDDIR = build
MANDIR   = $(BUILDDIR)/man
VERSION  = $(shell cat ../VERSION)
TODAY    = $(shell date +%Y-%m-%d)

PANDOC   = pandoc --standalone --from gfm+definition_lists --to man \
           --metadata date=$(TODAY) --metadata footer=$(VERSION) --metadata header=RPi-Monitor

# $(call manpage,name,section,description,page)
# The pages have no NAME section (needed by whatis/apropos), so it is piped in first.
define manpage
	printf '# NAME\n\n$(1) - $(3)\n' | $(PANDOC) --metadata title=$(1) --metadata section=$(2) \
	  - source/$(4).md --output $(MANDIR)/$(1).$(2)
endef

.PHONY: html
html:
	mdbook build

.PHONY: man
man:
	mkdir -p $(MANDIR)
	$(call manpage,rpimonitor,1,RPi-Monitor program,13_execution)
	$(call manpage,rpimonitor-daemon.conf,5,RPi-Monitor daemon configuration,21_daemon)
	$(call manpage,rpimonitor-data.conf,5,RPi-Monitor configuration,22_extraction)
	$(call manpage,rpimonitor-alert.conf,5,RPi-Monitor alert configuration,26_alert)

.PHONY: clean
clean:
	rm -rf $(BUILDDIR)
