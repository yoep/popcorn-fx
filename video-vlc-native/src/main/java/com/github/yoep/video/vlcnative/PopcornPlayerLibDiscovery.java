package com.github.yoep.video.vlcnative;

import com.sun.jna.NativeLibrary;
import org.springframework.util.StringUtils;

public class PopcornPlayerLibDiscovery {
    private static final String POPCORN_PLAYER_LIB_PATH_PROPERTY = "popcorn-player.path";
    private static final String[] DIRECTORIES = {
            System.getProperty("user.dir"),
            "assets",
            "assets/linux",
            "assets/linux-arm",
            "assets/macosx",
            "assets/windows"
    };

    public PopcornPlayerLibDiscovery() {
        initializePropertyPath();
        initializeWellKnowSearchPaths();
    }

    private void initializePropertyPath() {
        var popcornPlayerPath = System.getProperty(POPCORN_PLAYER_LIB_PATH_PROPERTY);

        if (!StringUtils.isEmpty(popcornPlayerPath)) {
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
