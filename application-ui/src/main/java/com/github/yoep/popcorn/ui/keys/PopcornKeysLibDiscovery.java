package com.github.yoep.popcorn.ui.keys;

import com.sun.jna.NativeLibrary;
import org.springframework.stereotype.Component;
import org.springframework.util.StringUtils;

@Component
public class PopcornKeysLibDiscovery {
    public static final String LIBRARY_NAME = "PopcornKeys";

    private static final String POPCORN_KEYS_LIB_PATH_PROPERTY = "popcorn-keys.path";
    private static final String[] DIRECTORIES = {
            System.getProperty("user.dir"),
            "assets",
            "assets/linux",
            "assets/linux-arm",
            "assets/macosx",
            "assets/windows"
    };

    public PopcornKeysLibDiscovery() {
        initializePropertyPath();
        initializeWellKnowSearchPaths();
    }

    /**
     * Verify if the Popcorn Keys library can be found.
     *
     * @return Returns true if the library was found, else false.
     */
    public boolean libraryFound() {
        try {
            PopcornKeysLib.popcorn_keys_release(null);
            return true;
        } catch (UnsatisfiedLinkError ex) {
            return false;
        }
    }

    private void initializePropertyPath() {
        var popcornKeysPath = System.getProperty(POPCORN_KEYS_LIB_PATH_PROPERTY);

        if (!StringUtils.isEmpty(popcornKeysPath)) {
            NativeLibrary.addSearchPath(LIBRARY_NAME, popcornKeysPath);
        }
    }

    private static void initializeWellKnowSearchPaths() {
        for (String directory : DIRECTORIES) {
            NativeLibrary.addSearchPath(LIBRARY_NAME, directory);
        }
    }
}
