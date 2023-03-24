# Application launch options

The following launch options can be used as startup arguments.

| Option                       | Description                                                     |
|------------------------------|-----------------------------------------------------------------|
| disable-vlc-video-player     | Disable the VLC video player from being activated.              |
| disable-youtube-video-player | Disabled the youtube player from being activated.               |
| disable-javafx-video-player  | Disabled the JavaFX player from being activated.                |
| disable-chromecast-player    | Disabled the chromecast player from being loaded.               |
| disable-mouse                | Permanently hides the mouse from the application.               |
| kiosk                        | Activate the kiosk mode (use alt+f4 to close the application).  |
| tv                           | Activate the tv mode (easier to use UI but less functionality). |
| maximized                    | Maximize the window on startup.                                 |
| insecure                     | Allow insecure connections                                      |

## Java launch options

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