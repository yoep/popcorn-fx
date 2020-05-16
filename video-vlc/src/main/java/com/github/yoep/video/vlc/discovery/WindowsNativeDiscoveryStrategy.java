package com.github.yoep.video.vlc.discovery;

import lombok.EqualsAndHashCode;
import lombok.ToString;
import uk.co.caprica.vlcj.binding.LibC;
import uk.co.caprica.vlcj.binding.RuntimeUtil;

@ToString
@EqualsAndHashCode(callSuper = true)
public class WindowsNativeDiscoveryStrategy extends DirectoryProviderDiscoveryStrategy {
    private static final String[] FILENAME_PATTERNS = new String[]{
            "libvlc\\.dll",
            "libvlccore\\.dll"
    };

    private static final String[] PLUGIN_PATH_FORMATS = new String[]{
            "%s\\plugins",
            "%s\\vlc\\plugins"
    };

    public WindowsNativeDiscoveryStrategy() {
        super(FILENAME_PATTERNS, PLUGIN_PATH_FORMATS);
    }

    @Override
    public boolean supported() {
        return RuntimeUtil.isWindows();
    }

    @Override
    protected boolean setPluginPath(String pluginPath) {
        return LibC.INSTANCE._putenv(String.format("%s=%s", PLUGIN_ENV_NAME, pluginPath)) == 0;
    }
}
