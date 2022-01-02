package com.github.yoep.player.qt.discovery;

import com.github.yoep.player.qt.PopcornPlayerLibRuntime;
import com.sun.jna.NativeLibrary;
import org.springframework.util.StringUtils;

public class QtPlayerLibDiscovery {
    private static final String POPCORN_PLAYER_LIB_PATH_PROPERTY = "popcorn-player.path";
    private static final String[] DIRECTORIES = {
            System.getProperty("user.dir"),
            "assets",
            "assets/linux",
            "assets/linux-arm",
            "assets/mac",
            "assets/windows"
    };

    public QtPlayerLibDiscovery() {
        initializePropertyPath();
        initializeWellKnowSearchPaths();
    }

    private void initializePropertyPath() {
        var popcornPlayerPath = System.getProperty(POPCORN_PLAYER_LIB_PATH_PROPERTY);

        if (!StringUtils.hasText(popcornPlayerPath)) {
            NativeLibrary.addSearchPath(PopcornPlayerLibRuntime.getLibraryName(), popcornPlayerPath);
        }
    }

    private static void initializeWellKnowSearchPaths() {
        var libraryName = PopcornPlayerLibRuntime.getLibraryName();

        for (String directory : DIRECTORIES) {
            NativeLibrary.addSearchPath(libraryName, directory);
        }
    }
}
