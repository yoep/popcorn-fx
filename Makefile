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
EXTENSION := dll
PROFILE := windows
else ifeq ($(SYSTEM),Darwin)
EXTENSION := dylib
PROFILE := macosx
else
EXTENSION := so
PROFILE := linux
endif

## Define all rust libraries and resource directories
LIBRARIES := popcorn-fx

prerequisites: ## Install the requirements for the application
	$(info Installing Cargo plugins)
	@cargo install cbindgen
	@cargo install cargo-nextest
	@cargo install grcov
	@mvn -B -P$(PROFILE) -pl torrent-frostwire clean

clean: prerequisites ## Clean the output
	$(info Cleaning output directories)
	@cargo clean
	@mvn -B clean

test-cargo: prerequisites ## The test cargo section of the application
	$(info Running cargo tests)
	@cargo nextest run

test: prerequisites test-cargo ## Test the application code
	$(info Running maven tests)
	@mvn -B verify -P$(PROFILE)

build-cargo: ## Build the rust part of the application
	$(info Using lib extension: $(EXTENSION))
	$(info Building cargo packages)
	@cargo build

build-cargo-release: ## Build the rust part of the application in release profile
	$(info Using lib extension: $(EXTENSION))
	$(info Building cargo packages)
	@cargo test && cargo build --release

## Copy the cargo libraries to the java resources
ifeq ($(SYSTEM),Windows)
lib-copy: build-cargo $(RESOURCE_DIRECTORIES)
	$(info Copying libraries to java resources)
	@$(foreach file,$(LIBRARIES),xcopy ".\target\debug\$(subst -,_,$(file)).$(EXTENSION)" ".\assets\$(PROFILE)\" /R /I /F /Y && ) echo.
else
lib-copy: build-cargo $(RESOURCE_DIRECTORIES)
	$(info Copying libraries to java resources)
	@$(foreach file,$(LIBRARIES),cp "target/debug/lib$(subst -,_,$(file)).$(EXTENSION)" "assets/$(PROFILE)/";)
endif

build-java: lib-copy ## Build the java part of the application
	$(info Building java)
	@mvn -B compile -P$(PROFILE)

build: prerequisites build-cargo lib-copy build-java ## Build the application

package: prerequisites build ## Package the application for distribution
	@mvn -B install -DskipTests -DskipITs -P$(PROFILE)

release: prerequisites build-cargo-release cargo-lib-copy ## Release a new version of the application
	$(info Starting maven gitflow release)
	@mvn -B -P$(PROFILE) gitflow:release

