package com.github.yoep.popcorn.ui.keys;

import com.sun.jna.NativeLibrary;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Component;
import org.springframework.util.StringUtils;

@Slf4j
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
            log.trace("Trying to load the Popcorn Keys library");
            PopcornKeysLib.popcorn_keys_release(null);
            return true;
        } catch (UnsatisfiedLinkError ex) {
            log.trace("Failed to load the Popcorn Keys library, " + ex.getMessage(), ex);
            return false;
        }
    }

    private void initializePropertyPath() {
        var popcornKeysPath = System.getProperty(POPCORN_KEYS_LIB_PATH_PROPERTY);

        if (StringUtils.hasText(popcornKeysPath)) {
            NativeLibrary.addSearchPath(LIBRARY_NAME, popcornKeysPath);
        }
    }

    private static void initializeWellKnowSearchPaths() {
        for (String directory : DIRECTORIES) {
            NativeLibrary.addSearchPath(LIBRARY_NAME, directory);
        }
    }
}
