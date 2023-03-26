.PHONY: cargo mvn help clean test build-cargo build

## Detect the OS
ifeq ($(OS),Windows_NT)
SYSTEM := Windows
ARCH := $(PROCESSOR_ARCHITECTURE)
else
SYSTEM := $(shell sh -c 'uname 2>/dev/null || echo Unknown')
ARCH := $(shell uname -m)
endif
$(info Detected OS: $(SYSTEM))
$(info Detected arch: $(ARCH))

## Set the system information
ifeq ($(SYSTEM),Windows)
LIBRARY := "popcorn_fx.dll"
PROFILE := windows
ASSETS := windows
PYTHON := python.exe
else ifeq ($(SYSTEM),Darwin)
LIBRARY := "libpopcorn_fx.dylib"
PROFILE := macosx
ASSETS := mac
PYTHON := python3
else
LIBRARY := "libpopcorn_fx.so"
PROFILE := linux
ASSETS := linux
PYTHON := python3
endif

prerequisites: ## Install the requirements for the application
	$(info Updating rust)
	@rustup update
	$(info Installing Cargo plugins)
	@cargo install cbindgen
	@cargo install cargo-nextest
	@cargo install cargo-llvm-cov
	@cargo install grcov
	@mvn -B -P$(PROFILE) -pl torrent-frostwire clean

bump-dependencies: ## Install required bump dependencies
	@$(PYTHON) -m pip install --upgrade pip
	@pip install bump2version --user

bump-%: bump-dependencies ## Bump the (major, minor, patch) version of the application
	@bumpversion $*

clean: prerequisites ## Clean the output
	$(info Cleaning output directories)
	@cargo clean
	@mvn -B clean

test-cargo: prerequisites ## The test cargo section of the application
	$(info Running cargo tests)
	@cargo llvm-cov --lcov --features ffi --output-path target/lcov.info nextest

cov-cargo: prerequisites ## Test coverage of the cargo section as std output
	$(info Running cargo tests)
	@cargo llvm-cov nextest --features ffi

test-java: prerequisites ## The test java section of the application
	$(info Running maven tests)
	@mvn -B clean verify -P$(PROFILE)

test: prerequisites test-java test-cargo ## Test the application code

build-cargo: ## Build the rust part of the application
	$(info Using lib extension: $(EXTENSION))
	$(info Building cargo packages)
	@cargo build --features ffi

build-cargo-release:  ## Build the rust part of the application in release profile
	$(info Using lib extension: $(EXTENSION))
	$(info Building cargo packages)
	@cargo build --release --features ffi

## Copy the cargo libraries to the java resources
lib-copy-%: build-cargo $(RESOURCE_DIRECTORIES)
	cp -v "./target/$*/$(LIBRARY)" "./assets/$(ASSETS)/"

lib-copy: lib-copy-debug ## The default lib-copy target

build-java: lib-copy-debug ## Build the java part of the application
	$(info Building java)
	@mvn -B compile -P$(PROFILE)

build-java-release: lib-copy-release ## Build the java part of the application
	$(info Building java)
	@mvn -B compile -P$(PROFILE)

build: prerequisites build-cargo build-java ## Build the application in debug mode

build-release: prerequisites build-cargo-release build-java-release ## Build the application in release mode (slower build time)

package: build-release ## Package the application for distribution
	@mvn -B install -DskipTests -DskipITs -P$(PROFILE)

release: bump-minor test-cargo build-release ## Release a new version of the application with increased minor

release-bugfix: bump-patch test-cargo build-release ## Release a patch of the application
