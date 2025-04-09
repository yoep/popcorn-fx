.PHONY: cargo mvn help clean test build-cargo build

## Set global variables
VERSION = 0.8.2
RUNTIME_VERSION = 21.0.3

ifeq ($(shell command -v jenv >/dev/null 2>&1 && echo "found"),found)
	JAVA_HOME = $(shell dirname $(shell dirname $(shell readlink -f $(shell jenv which jlink))))
endif

## Detect the OS
ifeq ($(OS),Windows_NT)
SYSTEM = Windows
ARCH = $(shell echo "$(PROCESSOR_ARCHITECTURE)" | tr '[:upper:]' '[:lower:]')
else
SYSTEM = $(shell sh -c 'uname 2>/dev/null || echo Unknown')
ARCH = $(shell uname -m | tr '[:upper:]' '[:lower:]')
endif
$(info Detected OS: $(SYSTEM))
$(info Detected ARCH: $(ARCH))
$(info Detected JAVA_HOME: $(JAVA_HOME))
$(info Detected JAVA version: $(shell java -version 2>&1 | awk -F '"' '/version/ {print $2}'))
$(info Detected Cargo version: $(shell cargo --version))
$(info Detected Rust version: $(shell rustc --version))

## Set the system information
ifeq ($(SYSTEM),Windows)
LIBRARY_EXECUTABLE := "libfx.exe"
EXECUTABLE := "popcorn-time.exe"
ASSETS := windows
PYTHON := python.exe

# check required software
ISCC_PATH := $(shell which iscc)
ifndef ISCC_PATH
   $(error "Inno Setup Compiler (iscc) is not installed")
endif

else ifeq ($(SYSTEM),Darwin)
LIBRARY_EXECUTABLE := "libfx"
EXECUTABLE := "popcorn-time"
ASSETS := mac
PYTHON := python3
else
LIBRARY_EXECUTABLE := "libfx"
EXECUTABLE := "popcorn-time"
ASSETS := debian
PYTHON := python3
endif

prerequisites-cargo:  ## Install the requirements for Cargo
	@echo Installing Cargo plugins
	@cargo install cbindgen
	@cargo install cargo-nextest
	@cargo install cargo-llvm-cov
	@cargo install grcov

prerequisites: prerequisites-cargo ## Install the requirements for the application

bump-dependencies: ## Install required bump dependencies
	@$(PYTHON) -m pip install --upgrade pip
	@pip install bump2version --user

bump-%: bump-dependencies ## Bump the (major, minor, patch) version of the application
	@bumpversion $*

clean: prerequisites ## Clean the output
	$(info Cleaning output directories)
	@cargo clean
	@mvn -B clean

test-cargo: prerequisites-cargo ## The test cargo section of the application
	$(info Running cargo tests)
	@cargo llvm-cov --lcov --output-path target/lcov.info nextest

cov-cargo: prerequisites ## Test coverage of the cargo section as std output
	$(info Running cargo tests)
	@cargo llvm-cov nextest

test-java: ## The test java section of the application
	$(info Running maven tests)
	@mvn -B clean verify

test: test-java test-cargo ## Test the application code

build-cargo: ## Build the rust part of the application
	$(info Using lib extension: $(EXTENSION))
	$(info Building cargo packages)
	@cargo build --profile dev

build-cargo-debug: build-cargo ## The alias for build-cargo which build the rust part of the application in debug profile

build-cargo-release:  ## Build the rust part of the application in release profile
	$(info Using lib extension: $(EXTENSION))
	$(info Building cargo packages)
	@cargo build --release

lib-copy-%: build-cargo-% $(RESOURCE_DIRECTORIES) ## Copy the cargo libraries to the assets resources
	@cp -v "./target/$*/$(LIBRARY_EXECUTABLE)" "./assets/$(ASSETS)/"
	@cp -v "./target/$*/$(EXECUTABLE)" "./assets/$(ASSETS)/"

lib-copy: lib-copy-debug ## The default lib-copy target

build-java: lib-copy-debug ## Build the java part of the application
	$(info Building java)
	@mvn -B compile

build-java-release: lib-copy-release ## Build the java part of the application
	$(info Building java)
	@mvn -B compile

build: prerequisites build-cargo build-java ## Build the application in debug mode

build-release: prerequisites build-cargo-release build-java-release ## Build the application in release mode (slower build time)

# Target: package-clean
# Description: Remove the old package target directory if it exists.
#
# Usage:
#   make package-clean
package-clean:
	@echo Cleaning installation package
	@rm -rf "./target/package"
	@rm -f "./target/*.tar.gz"

# Target: package-java
# Description: Package the java section of the application for distribution.
#
# Usage:
#   make package-java
package-java:
	@echo Packaging Java
	@mvn -B package -DskipTests -DskipITsQ

package-jre: ## Package the JRE runtime for distribution
	@echo Creating JRE bundle
	@rm -rf ./target/package/runtimes/*
	@APP_MODULES=$$(jdeps -q --ignore-missing-deps --print-module-deps --module-path "${JAVA_HOME}/jmods" application/target/popcorn-time.jar) &&\
		echo "Bundling modules $${APP_MODULES}" &&\
		"${JAVA_HOME}/bin/jlink" --module-path="${JAVA_HOME}/jmods" --add-modules "$${APP_MODULES}" --output "./target/package/runtimes/${RUNTIME_VERSION}/jre" --no-header-files --no-man-pages --strip-debug --compress=2

package: package-clean build-release package-java package-jre ## Package the application for distribution
	@echo Copying exeutable and libraries
	@cp -v ./assets/${ASSETS}/*.${LIBRARY_EXTENSION} ./target/package/
	@cp -v ./target/release/${EXECUTABLE} ./target/package/
	@cp -v ./application/target/popcorn-time.jar ./target/package/

	@echo Creating installer
	@export VERSION=${VERSION}; ./assets/${ASSETS}/installer.sh

	@echo Creating runtime update
	@cd target/package/runtimes && tar -cvzf ../../patch_runtime_${RUNTIME_VERSION}_${ASSETS}_${ARCH}.tar.gz ${RUNTIME_VERSION}/*
	@rm -rf target/package/runtimes

	@echo Creating app update
	@cd target/package && tar -cvzf ../patch_app_${VERSION}_${ASSETS}_${ARCH}.tar.gz *

release: bump-minor test-cargo build-release ## Release a new version of the application with increased minor

release-bugfix: bump-patch test-cargo build-release ## Release a patch of the application
