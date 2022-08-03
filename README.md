# Popcorn FX

![Build](https://github.com/yoep/popcorn-fx/workflows/Build/badge.svg)
![Version](https://img.shields.io/github/v/tag/yoep/popcorn-fx?label=version)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![codecov](https://codecov.io/gh/yoep/popcorn-fx/branch/master/graph/badge.svg?token=A801IOOZAH)](https://codecov.io/gh/yoep/popcorn-fx)

Popcorn FX was based on the original Popcorn Desktop and Popcorn Android versions. Popcorn FX uses **Java**
and **OpenJFX**. This version was created to work with embedded devices such as the Raspberry PI, but it also works
on desktop environments such as **Linux, Windows & Mac**.

Popcorn FX has been tested with **Raspberry Pi 3B+ and 4**.

## Launch options

The following launch options can be used as startup arguments.

| Option                       | Description                                                     |
|------------------------------|-----------------------------------------------------------------|
| disable-vlc-video-player     | Disable the VLC video player from being activated.              |
| disable-youtube-video-player | Disabled the youtube player from being activated.               |
| disable-javafx-video-player  | Disabled the JavaFX player from being activated.                |
| disable-chromecast-player    | Disabled the chromecast player from being loaded.               |
| disable-qt-player            | Disabled the QT player from being loaded.                       |
| disable-keep-alive           | Disable the keep alive which sends periodic key events.         |
| disable-mouse                | Permanently hides the mouse from the application.               |
| force-arm-video-player       | Force the use of the arm video player.                          |
| big-picture                  | Activate the big picture mode.                                  |
| kiosk                        | Activate the kiosk mode (use alt+f4 to close the application).  |
| tv                           | Activate the tv mode (easier to use UI but less functionality). |
| maximized                    | Maximize the window on startup.                                 |

### Java launch options

### GC optimization

If you want to reduce the memory footprint of the application, it's recommended to add the following argument to the VM
options:

    -XX:+UseG1GC

This option is already added to the packaged executables
(which is not the case for the standalone JAR file).

### Virtual Keyboard

If you're running the application on a touch screen device, it's recommended to enabled the virtual keyboard. This can
be done by adding the following argument to the VM options:

    -Dcom.sun.javafx.virtualKeyboard=javafx 

## System Requirements

### Minimal

- CPU: 1.2GHz
- Memory: 500MB

## Known issues

### IntelliJ IDEA

IntelliJ adds by default the `javafx.base` and `javafx.graphics` to the modules of Java 9+. This might be causing issues
in Java 9 and above, as the `javafx.controls` and `javafx.fxml` are missing from the modules causing
an `IllegalAccessException` when trying to run the application.

Add the following options to the `VM Options` in the run configuration of IntelliJ to fix this issue.

    -p "<PATH TO JAVAFX SDK>\lib" --add-modules javafx.controls,javafx.fxml,javafx.graphics,javafx.media,javafx.web,javafx.swing

You need to add `javafx.swing` in the modules list if you want to use ScenicView for inspecting the JavaFX UI. If you
don't do this, the application will crash when trying to attach to the Java process that is running JavaFX.

### White box glitch

Add the following VM option if you're experiencing white boxes in the UI. This option is enabled by default on the
pre-built versions.

    -Dprism.dirtyopts=false

## Development

To run the application from source code locally, add the following VM options.

    -Djava.library.path=assets/<<OS>>:${PATH}
    --add-modules javafx.controls,javafx.fxml,javafx.graphics,javafx.media,javafx.web,javafx.swing

### Runtime

It's advised to use `PopcornTimeApplication` as main entry during development. The reason behind this is that the
PopcornTimeStarter is only used for the fat jar packaging.

Within the runtime configuration, make sure that the classpath is set to the `application` module.

![Application module setup](https://i.imgur.com/EVDQLmS.png)

### Dependencies

The following dependencies are required for development.

- Java 11+
- OpenJFX 17+
- Make
- Rust

### Building player QT

The module `player-qt` makes use of a native VLC player build on top of `QT5+` for video playbacks. This is used
for the Raspberry Pi video playbacks to increase the performance as the JavaFX rendering is too heavy for video
playbacks.

To install the native VLC player, run the following maven command:

    mvn clean install -Pcmake -Dqt.compiler="QT_COMPILER_LOCATION" -Dcmake.dir="CMAKE_INSTALLATION_DIR"

## Screenshots

#### Desktop mode

![Desktop preview](https://i.imgur.com/hkmMGDb.png)

![Desktop Movie details](https://i.imgur.com/Rz6ABUu.png)

![Desktop watchlist](https://i.imgur.com/bG5MiKF.png)

#### TV mode

![TV preview](https://i.imgur.com/QHQQKQk.png)

![TV Movie details](https://i.imgur.com/FD0Hp3o.png)
