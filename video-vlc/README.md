# VLC Video player

The VLC video player module makes use of the VLCJ library to render the VLC 
content into a `WritableImage` which displayed within the JavaFX scene.

## Dependencies

The VLC video player module depends on the following libraries:

- `uk.co.caprica:vlcj`

## Conditions

The VLC video player is only activated when the following conditions match:

- VLC installation has been found on the System
- Current CPU architecture is non-ARM
- VLC video player has not been disabled (through option arguments)

If the all of the above conditions match, the VLC video player module might
be used during a video playerback.
