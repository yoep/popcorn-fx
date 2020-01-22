# Popcorn Time Desktop Java

Popcorn Time Desktop application based on the original Popcorn Time Desktop and Popcorn Time Android versions.
This version was created to work with embedded devices such as the Raspberry PI.

## Recommended startup options

### Disable AWT freeze

Add the following VM option argument to prevent AWT from freezing the IDE 
when running in debug mode.

    -Dsun.awt.disablegrab=true

### Better font rendering

Add the following argument to the VM options when you're experiencing blurry text.

    -Dprism.lcdtext=false

### GC optimization

If you want to reduce the memory footprint of the application, 
it's recommended to add the following argument to the VM options:

    -XX:+UseG1GC

### Virtual Keyboard

If you're running the application on a touch screen device, 
it's recommended to enabled the virtual keyboard.
This can be done by adding the following argument to the VM options:

    -Dcom.sun.javafx.virtualKeyboard=javafx 

## System Requirements

### Minimal

- Java 11+
- OpenJFX 13+
- CPU: 1.5GHz
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

### White box glitch

Add the following VM option if you're experiencing white boxes in the UI.

    -Dprism.dirtyopts=false

## Features

### v1.0.0

### Upcoming features

- Update torrent buffering when seeking through the video
- Resume video on last view time
- Implement watchlist
- Implement torrent collection
- Add ability to store a pasted magnet link
- Add loading indicator to the player
- Update video time slider to show video loading progress (custom control)
- Trakt.tv integration
- Continue to watch show with next episode
- Convert project from unnamed to modules setup for smaller bundled JRE