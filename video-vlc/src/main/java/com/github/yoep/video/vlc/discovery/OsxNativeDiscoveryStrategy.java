package com.github.yoep.video.vlc.discovery;

import com.github.yoep.video.vlc.LibC;
import com.github.yoep.video.vlc.RuntimeUtil;
import com.sun.jna.NativeLibrary;
import lombok.EqualsAndHashCode;
import lombok.ToString;

@ToString
@EqualsAndHashCode(callSuper = true)
public class OsxNativeDiscoveryStrategy extends DirectoryProviderDiscoveryStrategy {

    private static final String[] FILENAME_PATTERNS = new String[]{
            "libvlc\\.dylib",
            "libvlccore\\.dylib"
    };

    /**
     * Format string to prepare the plugin path environment variable value.
     */
    private static final String[] PLUGIN_PATH_FORMATS = new String[]{
            "%s/../plugins"
    };

    public OsxNativeDiscoveryStrategy() {
        super(FILENAME_PATTERNS, PLUGIN_PATH_FORMATS);
    }

    @Override
    public boolean supported() {
        return RuntimeUtil.isMac();
    }

    @Override
    public boolean onFound(String path) {
        forceLoadLibVlcCore(path);
        return true;
    }

    /**
     * On later versions of OSX, it is necessary to force-load libvlccore before libvlc, otherwise libvlc will fail to
     * load.
     *
     * @param path
     */
    private void forceLoadLibVlcCore(String path) {
        NativeLibrary.addSearchPath(RuntimeUtil.getLibVlcCoreLibraryName(), path);
        NativeLibrary.getInstance(RuntimeUtil.getLibVlcCoreLibraryName());
    }

    @Override
    protected boolean setPluginPath(String pluginPath) {
        return LibC.INSTANCE.setenv(PLUGIN_ENV_NAME, pluginPath, 1) == 0;
    }

}
