.PHONY: cargo mvn help clean test build-cargo build

## Detect the OS
ifeq ($(OS),Windows_NT)
SYSTEM := Windows
else
SYSTEM := $(shell sh -c 'uname 2>/dev/null || echo Unknown')
endif

## Set the system information
ifeq ($(SYSTEM),Windows)
EXTENSION := dll
OS_RESOURCE_DIR := win32-x86-64
PROFILE := windows
else ifeq ($(SYSTEM),Darwin)
EXTENSION := dylib
OS_RESOURCE_DIR := darwin-x86-64
PROFILE := macosx
else
EXTENSION := so
OS_RESOURCE_DIR := debian-x86-64
PROFILE := linux
endif

## Define all rust libraries and resource directories
LIBRARIES := application application-backend application-platform
RESOURCE_DIRECTORIES := $(addsuffix /src/main/resources/$(OS_RESOURCE_DIR)/, $(LIBRARIES))

## Create the resource directories if needed for java
ifeq ($(SYSTEM),Windows)
%/src/main/resources/$(OS_RESOURCE_DIR)/:
	$(info Creating directory $@)
	mkdir "$@"
else
%/src/main/resources/$(OS_RESOURCE_DIR)/:
	$(info Creating directory $@)
	mkdir -p $@
endif

prerequisites: ## Install the requirements for the application
	$(info Installing Cargo plugins)
	@cargo install cbindgen

clean: prerequisites ## Clean the output
	$(info Cleaning output directories)
	@cargo clean
	@mvn -B clean

test: ## Test the application code
	$(info Running cargo tests)
	@cargo test

	$(info Running maven tests)
	@mvn -B clean verify -P$(PROFILE)

build-cargo: $(RESOURCE_DIRECTORIES) ## Build the rust part of the application
	$(info Current OS: $(OS))
	$(info Using lib extension: $(EXTENSION))
	$(info Building cargo packages)
	@cargo build

## Copy the cargo libraries to the java resources
ifeq ($(SYSTEM),Windows)
cargo-lib-copy: build-cargo
	$(info Copying libraries to java resources)
	@$(foreach file,$(LIBRARIES),xcopy "target\\debug\\$(file).$(EXTENSION)" "$(file)\\src\\main\\resources\\$(OS_RESOURCE_DIR)\\" /f /y;)

else
cargo-lib-copy: build-cargo
	$(info Copying libraries to java resources)
	@$(foreach file,$(LIBRARIES),cp "target/debug/lib$(subst -,_,$(file)).$(EXTENSION)" "$(file)/src/main/resources/$(OS_RESOURCE_DIR)/";)
endif

build-java: cargo-lib-copy ## Build the java part of the application
	$(info Building java)
	@mvn -B clean compile -P$(PROFILE)

build: prerequisites build-cargo cargo-lib-copy build-java ## Build the application

package: prerequisites build ## Package the application for distribution
	@mvn -B install -P$(PROFILE)


