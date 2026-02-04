PREFIX ?= /usr
DATADIR ?= $(PREFIX)/share

build:
	cargo build --release -p ibus-akaza -p akaza-conf -p akaza-dict

all: build
	$(MAKE) -C ibus-akaza all

install: install-resources install-model
	install -m 0755 target/release/ibus-akaza $(PREFIX)/bin/
	install -m 0755 target/release/akaza-conf $(PREFIX)/bin/
	install -m 0755 target/release/akaza-dict $(PREFIX)/bin/
	$(MAKE) -C ibus-akaza install

install-model:
	mkdir -p $(DATADIR)/akaza/model/default/
	curl -L https://github.com/akaza-im/akaza-default-model/releases/latest/download/akaza-default-model.tar.gz | \
		tar xzv --strip-components=1 -C $(DATADIR)/akaza/model/default/

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

.PHONY: all build install install-model install-resources clean \
	docker-test-build docker-test docker-test-unit docker-test-integration docker-test-e2e docker-test-shell
