PREFIX ?= /usr
DATADIR ?= $(PREFIX)/share

all:
	$(MAKE) -C ibus-akaza all

install: install-resources
	$(MAKE) -C ibus-akaza install

install-resources:
	install -m 0644 -v -D -t $(DATADIR)/akaza/romkan romkan/*
	install -m 0644 -v -D -t $(DATADIR)/akaza/keymap keymap/*

clean:
	cargo clean
	$(MAKE) -C ibus-akaza clean

# Docker test targets
docker-test-build:
	docker compose -f docker-compose.test.yml build

docker-test:
	docker compose -f docker-compose.test.yml run --rm test test

docker-test-unit:
	docker compose -f docker-compose.test.yml run --rm test test-unit

docker-test-integration:
	docker compose -f docker-compose.test.yml run --rm test test-integration

docker-test-e2e:
	docker compose -f docker-compose.test.yml run --rm test test-e2e

docker-test-shell:
	docker compose -f docker-compose.test.yml run --rm test bash

.PHONY: all install install-resources clean \
	docker-test-build docker-test docker-test-unit docker-test-integration docker-test-e2e docker-test-shell

