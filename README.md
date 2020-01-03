# Popcorn Time Desktop Java

Popcorn Time Desktop application based on the original Popcorn Time Desktop and Popcorn Time Android versions.
This version was created to work with embedded devices such as the Raspberry PI.

## Startup options

    -Dsun.awt.disablegrab=true -XX:+UseG1GC -Dprism.lcdtext=false -Xmx1G
    
## IntelliJ IDEA

IntelliJ adds by default the `javafx.base` and `javafx.graphics` to the modules of Java 9+.
This might be causing issues in Java 9 and above, as the `javafx.controls` and `javafx.fxml` are 
missing from the modules causing an `IllegalAccessException` when trying to run the application.

Add the following options to the `VM Options` in the run configuration of IntelliJ to fix this issue. 

    -p "<PATH TO JAVAFX SDK>\lib" --add-modules javafx.controls,javafx.fxml,javafx.graphics,javafx.media,javafx.web,javafx.swing

## White box glitch

Add the following VM option when your experiencing white boxes in the UI.

    -Dprism.dirtyopts=false

## Virtual Keyboard

Add the following VM option to enable the virtual keyboard

    -Dcom.sun.javafx.virtualKeyboard=javafx 

## TODO features

### Must

- Set default subtitle when loading the details or playing the media
- Update torrent buffering when seeking through the video

### Should

- Resume video on last view time
- Paste magnet links in the application
- Implement watchlist
- Implement torrent collection

### Could

- Add loading indicator to the player
- Update video time slider to show video loading progress (custom control)
- Trakt.tv integration