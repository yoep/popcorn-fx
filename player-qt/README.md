# QT player

The QT player, also known as the Popcorn Player, make use of Qt5+ for
video playbacks. This module is created for ARM devices as the rendering through
JavaFX is too slow.

## Dependencies

The following libraries are required for development of this module:

- Qt5+

## Development

Make sure you have the following dependencies available:

- g++-11
- Qt5+ SDK

Run the cmake project with the bundled QT binaries:

```shell
cmake -DCMAKE_PREFIX_PATH=<<QT_INSTALL_DIRECTORY>> -DCMAKE_MODULE_PATH=<<QT_LIB_DIRECTORY>> /path-to-project-root
```

## Runtime

For this module to work/be activated, the following library should be available
somewhere within the library PATH:

- `libPopcornPlayer.so`
- `libPopcornPlayer.dylib`

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
