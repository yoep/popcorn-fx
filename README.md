# Popcorn Time Desktop Java

## Startup options

    -Dsun.awt.disablegrab=true -XX:+UseG1GC -Dprism.lcdtext=false -Xmx1G
    
## IntelliJ IDEA

IntelliJ adds by default the `javafx.base` and `javafx.graphics` to the modules of Java 9+.
This might be causing issues in Java 9 and above, as the `javafx.controls` and `javafx.fxml` are 
missing from the modules causing an `IllegalAccessException` when trying to run the application.

Add the following options to the `VM Options` in the run configuration of IntelliJ to fix this issue. 

    -p "<PATH TO JAVAFX SDK>\lib" --add-modules javafx.controls,javafx.fxml,javafx.swing
    
## TODO features

- Add video quality selection
- Add video quality to player
- Add stream stats to player
- Add settings
- Add favorites
- Mark as watched when the video is at 90%
- Resume video on last view time