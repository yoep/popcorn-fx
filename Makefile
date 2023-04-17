.PHONY: cargo mvn help clean test build-cargo build

## Set global variables
VERSION = 0.6.5
RUNTIME_VERSION = 17.0.6

ifeq ($(shell command -v jenv >/dev/null 2>&1 && echo "found"),found)
	JAVA_HOME = $(shell dirname $(shell dirname $(shell readlink -f $(shell jenv which jlink))))
endif

## Detect the OS
ifeq ($(OS),Windows_NT)
SYSTEM = Windows
ARCH = $(PROCESSOR_ARCHITECTURE)
else
SYSTEM = $(shell sh -c 'uname 2>/dev/null || echo Unknown')
ARCH = $(shell uname -m)
endif
$(info Detected OS: $(SYSTEM))
$(info Detected arch: $(ARCH))
$(info Detected JAVA_HOME: $(JAVA_HOME))

## Set the system information
ifeq ($(SYSTEM),Windows)
LIBRARY := "popcorn_fx.dll"
EXECUTABLE := "popcorn-time.exe"
PROFILE := windows
ASSETS := windows
PYTHON := python.exe
INSTALLER_COMMAND := powershell.exe -Command "iscc.exe /Otarget/ /Fpopcorn-time_${VERSION} \"./assets/windows/installer.iss\""
RUNTIME_COMPRESS_COMMAND := 7z a -tzip target/runtime_${RUNTIME_VERSION}_windows.zip target/package/runtimes/${RUNTIME_VERSION}/*

# check required software
ifeq ($(shell command -v iscc),)
   $(error "Inno Setup Compiler (iscc) is not installed")
endif

else ifeq ($(SYSTEM),Darwin)
LIBRARY := "libpopcorn_fx.dylib"
EXECUTABLE := "popcorn-time"
PROFILE := macosx
ASSETS := mac
PYTHON := python3
else
LIBRARY := "libpopcorn_fx.so"
EXECUTABLE := "popcorn-time"
PROFILE := linux
ASSETS := linux
PYTHON := python3
INSTALLER_COMMAND := dpkg-deb --build -Zgzip target/package target/popcorn-time_${VERSION}.deb
RUNTIME_COMPRESS_COMMAND := tar -czvf target/runtime_${RUNTIME_VERSION}_debian_x86_64.tar.gz target/package/runtimes/${RUNTIME_VERSION}/*
endif

prerequisites: ## Install the requirements for the application
	@echo Updating rust
	@rustup update
	@echo Installing Cargo plugins
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

build-cargo-debug: build-cargo ## The alias for build-cargo which build the rust part of the application in debug profile

build-cargo-release:  ## Build the rust part of the application in release profile
	$(info Using lib extension: $(EXTENSION))
	$(info Building cargo packages)
	@cargo build --release --features ffi

## Copy the cargo libraries to the java resources
lib-copy-%: build-cargo-% $(RESOURCE_DIRECTORIES)
	@cp -v "./target/$*/$(LIBRARY)" "./assets/$(ASSETS)/"
	@cp -v "./target/$*/$(EXECUTABLE)" "./assets/$(ASSETS)/"

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
	@echo Packaging Java
	@mvn -B package -P$(PROFILE) -DskipTests -DskipITs

	@echo Cleaning installation package
	@rm -rf "./target/package"

	@echo Creating JRE bundle
	@"${JAVA_HOME}/bin/jlink" --module-path="${JAVA_HOME}/jmods" --add-modules="ALL-MODULE-PATH" --output "./target/package/runtimes/${RUNTIME_VERSION}/jre" --no-header-files --no-man-pages --strip-debug --compress=2

	@echo Copying exeutable and libraries
	@cp -v ./target/release/${EXECUTABLE} ./target/package/
	@cp -v ./target/release/${LIBRARY} ./target/package/
	@cp -v ./application/target/popcorn-time.jar ./target/package/
	@if [ "$(SYSTEM)" = "Linux" ]; then export VERSION=${VERSION}; ./assets/linux/prepare-package.sh; fi

	@echo Creating installer
	${INSTALLER_COMMAND}

	@echo Creating runtime update
	${RUNTIME_COMPRESS_COMMAND}

release: bump-minor test-cargo build-release ## Release a new version of the application with increased minor

release-bugfix: bump-patch test-cargo build-release ## Release a patch of the application
