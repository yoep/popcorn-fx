# VLC native video player

The VLC native player, also known as the Popcorn Player, make use of Qt5+ for
video playbacks. This module is created for ARM devices as the rendering through
JavaFX is too slow.

## Dependencies

The following libraries are required for development of this module:

- Qt5+

## Conditions

The following conditions must match before the VLC native video player module
is activated:

- CPU architecture must be ARM
- The ARM video player is not disabled through the argument options

If the option `force-arm-video-player` is present, the VLC native player module
will always be loaded (even if the above conditions don't match).

## Runtime

For this module to work/be activated, the following library should be available
somewhere within the library PATH:

- `libPopcornPlayer.so`
