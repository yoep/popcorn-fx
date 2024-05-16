# Application launch options

The following launch options can be used as startup arguments.

| Option                      | Description                                                     | Default |
|-----------------------------|-----------------------------------------------------------------|---------|
| enable-vlc-video-player     | Enable the VLC video player from being activated.               | true    |
| enable-youtube-video-player | Enable the youtube player from being activated.                 | true    |
| enable-javafx-video-player  | Enable the JavaFX player from being activated.                  | true    |
| disable-chromecast-player   | Disabled the chromecast player from being loaded.               | false   |
| disable-mouse               | Permanently hides the mouse from the application.               | false   |
| kiosk                       | Activate the kiosk mode (use alt+f4 to close the application).  | false   |
| tv                          | Activate the tv mode (easier to use UI but less functionality). | false   |
| maximized                   | Maximize the window on startup.                                 |
| insecure                    | Allow insecure connections                                      |

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