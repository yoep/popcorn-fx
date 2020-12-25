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

## RPI boot config

It's recommended to enable full KMS on the Raspberry Pi to prevent screen tearing.
Find below the recommended boot configuration setup:

    [pi3]
    dtoverlay=vc4-kms-v3d
    over_voltage=4
    arm_freq=1500
    gpu_freq=500
    
    [pi4]
    dtoverlay=vc4-kms-v3d-pi4,noaudio
    max_framebuffers=2
    over_voltage=6
    arm_freq=2000
    v3d_freq=815
    gpu_freq=750
    
    [all]
    gpu_mem=256
