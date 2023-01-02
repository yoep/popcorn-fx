# Development

To run the application from source code locally, add the following VM options.

    -Djava.library.path=assets/<<OS>>:${PATH}
    --add-modules javafx.controls,javafx.fxml,javafx.graphics,javafx.media,javafx.web,javafx.swing

## Dependencies

The following dependencies are required for development:

- Java 17+
- OpenJFX 17+
- Make
- Rust/Cargo
- Qt5+
- Make (_optional, recommended to use_)

## Getting started

A [MakeFile](../Makefile) has been foreseen with some goals to get easily started.
Use one of the following provided goals.

Most of the targets also have a specific sub-task for Cargo or Java only.
_e.g.: build-cargo, build-java_

_The **cbingen** plugin for Cargo will always be installed through Make for almost all targets_

### clean

Clean all build target/output directories of Cargo and Java.

### test

Run all unit tests of the application.
This will start tests from Cargo and Java.

### build

Build the application.
This will start a build of cargo, the output libraries will be copied
to the correct directories within the java resources.

### package

Build the application and create an executable which can be distributed.

### release

Release a new version of the application.
This will build the Rust libraries in `release` profile (optimised) mode.
Afterwards, the maven `gitflow:release` target will be executed which will test, build & bump the version of the application.

## Running from an idea

It's advised to use `PopcornTimeApplication` as main entry during development. The reason behind this is that the
PopcornTimeStarter is only used for the fat jar packaging.

Within the runtime configuration, make sure that the classpath is set to the `application` module.

![Application module setup](https://i.imgur.com/EVDQLmS.png)