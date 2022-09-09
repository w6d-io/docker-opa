SHELL=/bin/bash -o pipefail

export PATH               := bin:${PATH}
export PWD                := $(shell pwd)
export NEXT_TAG           ?=


ifeq (darwin,$(shell rustup show | grep "Default host" | sed 's/.*\(darwin\).*/\1/'))
RUSTOS       = "darwin"
else
RUSTOS       = "linux"
endif

# Get cargo version
.PHONY: version
version:
	cargo --version
	$(info ************ OS SYSTEM IS $(RUSTOS) **********)

# Formats the code
.PHONY: format
format:
	cargo fmt

.PHONY: changelog
changelog:
	git-chglog -o CHANGELOG.md --next-tag $(NEXT_TAG)

# Run test
.PHONY: test
test:
	cargo test

.PHONY: bin/goreadme
bin/goreadme:
	cargo install cargo-readme

# Create readme
.PHONY: readme
readme: bin/goreadme
	cargo readme > README.md

# Create documentation
.PHONY: doc
doc:
	cargo doc

.PHONY: kratos
KRATOS_BINARY = $(shell pwd)/bin/kratos
KRATOS_TAR.GZ = $(shell pwd)/bin/kratos.tar.gz
SCRIPTBASH = $(shell pwd)/makefile.sh
GOBIN = $(shell pwd)/bin

ifeq (darwin,$(shell rustup show | grep "Default host" | sed 's/.*\(darwin\).*/\1/'))
KRATOS_BINARY_URL=https://github.com/ory/kratos/releases/download/v0.10.1/kratos_0.10.1-macOS_sqlite_64bit.tar.gz
else
KRATOS_BINARY_URL=https://github.com/ory/kratos/releases/download/v0.10.1/kratos_0.10.1-linux_sqlite_64bit.tar.gz
endif
kratos: ##init kratos
ifeq (,$(wildcard $(KRATOS_BINARY)))
	mkdir -p $(GOBIN)
	wget $(KRATOS_BINARY_URL) -O $(KRATOS_TAR.GZ)
	tar -xf $(KRATOS_TAR.GZ) -C $(GOBIN)
	chmod +x $(KRATOS_BINARY)
	chmod +x $(SCRIPTBASH)
	mkdir -p /var/lib/sqlite
	$(SCRIPTBASH) config &
else
	$(info ************ BINARY ALREADY EXIST **********)
endif

start: ## if binary file cannot execute verify the KRATOS_BINARY_URL OS (default linux or macosx)
	nohup $(KRATOS_BINARY) serve --dev -c $(GOBIN)/kratos.yml &

fake:  ## create fake data set for kratos
	curl -d @$(GOBIN)/testscope -X POST http://localhost:4434/admin/identities | jq -r '.'
	curl -d @$(GOBIN)/testperson -X POST http://localhost:4434/admin/identities | jq -r '.'
	curl -d @$(GOBIN)/testorg -X POST http://localhost:4434/admin/identities | jq -r '.'

stop:
ifeq (darwin,$(shell rustup show | grep "Default host" | sed 's/.*\(darwin\).*/\1/'))
	lsof -i -P | grep 4434 | sed -e 's/.*kratos     *//' -e 's#/.*##' | sed 's/ .*//' | xargs kill
else
	netstat -lnp | grep 4434 | sed -e 's/.*LISTEN *//' -e 's#/.*##' | xargs kill
endif

clean:
	rm -rf bin
