# Raspberry Pi

## RPI boot config

It's recommended to enable full KMS on the Raspberry Pi to prevent screen tearing. Find below the recommended boot configuration setup:

```shell
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
```