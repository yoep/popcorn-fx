# Popcorn Time Desktop JavaFX

Popcorn Time Desktop JavaFX is based on the original Popcorn Time Desktop and Popcorn Time Android versions.
Popcorn Time Desktop JavaFX uses **Java 11+** and **OpenJFX 14**.
This version was created to work with embedded devices such as the Raspberry PI, 
but it also works on desktop environments such as **Linux & Windows** (not tested on Mac).

Popcorn Time Desktop Java has been tested with **Raspberry Pi 3B+ and 4**.

## Launch options

The following launch options can be used as startup arguments.

Option                          | Description
---                             | ---
disable-arm-video-player        | Disable the arm video player from being activated.
disable-youtube-video-player    | Disabled the youtube player from being activated.
big-picture                     | Activate the big picture mode.
kiosk                           | Activate the kiosk mode (use alt+f4 to close the application).
tv                              | Activate the tv mode (easier to use UI but less functionality).

### Java launch options

### GC optimization

If you want to reduce the memory footprint of the application, 
it's recommended to add the following argument to the VM options:

    -XX:+UseG1GC
    
This option is already added to the packaged executables 
(which is not the case for the standalone JAR file).

### Virtual Keyboard

If you're running the application on a touch screen device, 
it's recommended to enabled the virtual keyboard.
This can be done by adding the following argument to the VM options:

    -Dcom.sun.javafx.virtualKeyboard=javafx 

## System Requirements

### Minimal

- CPU: 1.2GHz
- Memory: 500MB

### Recommended

- VLC media player

## Known issues  

### IntelliJ IDEA

IntelliJ adds by default the `javafx.base` and `javafx.graphics` to the modules of Java 9+.
This might be causing issues in Java 9 and above, as the `javafx.controls` and `javafx.fxml` are 
missing from the modules causing an `IllegalAccessException` when trying to run the application.

Add the following options to the `VM Options` in the run configuration of IntelliJ to fix this issue. 

    -p "<PATH TO JAVAFX SDK>\lib" --add-modules javafx.controls,javafx.fxml,javafx.graphics,javafx.media,javafx.web,javafx.swing

You need to add `javafx.swing` in the modules list if you want to use ScenicView for inspecting the JavaFX UI.
If you don't do this, the application will crash when trying to attach to the Java process that is running JavaFX.

### White box glitch

Add the following VM option if you're experiencing white boxes in the UI.

    -Dprism.dirtyopts=false

## Features

### v1.0.0

- List, filter & play movies
- List, filter & play shows
- Select video quality
- Select video subtitle
- Increase/decrease subtitle offset
- Mark media as watched
- Add media to favorites
- Paste & store magnet link
- Trakt.tv integration
- Dynamic torrent buffering when seeking through video
- Resume video from last known timestamp
- Torrent collection
- Big-picture & kiosk mode
- Watchlist

### Upcoming features

- Update video time slider to show video loading progress (custom control)
- Continue to watch show with next episode
- Convert project from unnamed to modules setup for smaller bundled JRE
- Add option for custom subtitle files
- TV remote support