package com.github.yoep.video.vlc.discovery;

import com.github.yoep.video.vlc.LibC;
import com.sun.jna.Platform;
import lombok.EqualsAndHashCode;
import lombok.ToString;

@ToString
@EqualsAndHashCode(callSuper = true)
public class LinuxNativeDiscoveryStrategy extends DirectoryProviderDiscoveryStrategy {
    private static final String[] FILENAME_PATTERNS = new String[]{
            "libvlc\\.so(?:\\.\\d)*",
            "libvlccore\\.so(?:\\.\\d)*"
    };

    private static final String[] PLUGIN_PATH_FORMATS = new String[]{
            "%s/plugins",
            "%s/vlc/plugins"
    };

    public LinuxNativeDiscoveryStrategy() {
        super(FILENAME_PATTERNS, PLUGIN_PATH_FORMATS);
    }

    @Override
    public boolean supported() {
        return Platform.isLinux();
    }

    @Override
    protected boolean setPluginPath(String pluginPath) {
        return LibC.INSTANCE.setenv(PLUGIN_ENV_NAME, pluginPath, 1) == 0;
    }

}
