.PHONY: cargo mvn help clean test build-cargo build

## Detect the OS
ifeq ($(OS),Windows_NT)
SYSTEM := Windows
else
SYSTEM := $(shell sh -c 'uname 2>/dev/null || echo Unknown')
endif

ifeq ($(SYSTEM),Windows)
EXTENSION := dll
OS_RESOURCE_DIR := win32-x86-64
endif

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

clean: ## Clean the output
	$(info Cleaning output directories)
	@cargo clean
	@mvn -B clean

test: ## Test the application code
	$(info Running cargo tests)
	@cargo test

	$(info Running maven tests)
	@mvn -B clean test

build-cargo: $(RESOURCE_DIRECTORIES) ## Build the application
	$(info Current OS: $(OS))
	$(info Using lib extension: $(EXTENSION))
	$(info Building cargo packages)
	@cargo build

## Copy the cargo libraries to the java resources
ifeq ($(SYSTEM),Windows)
cargo-lib-copy: build-cargo
	$(foreach file,$(LIBRARIES),xcopy "target\debug\$(file).$(EXTENSION)" "$(file)\src\main\resources\$(OS_RESOURCE_DIR)\" /f /y;)

else
cargo-lib-copy: build-cargo

endif

build: build-cargo cargo-lib-copy


