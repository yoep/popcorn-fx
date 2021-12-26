package com.github.yoep.popcorn.backend.settings;

import lombok.AccessLevel;
import lombok.NoArgsConstructor;

import java.io.File;

@NoArgsConstructor(access = AccessLevel.PRIVATE)
public class SettingsDefaults {
    public static final String APP_DIR = getDefaultAppDirLocation();

    private static String getDefaultAppDirLocation() {
        return System.getProperty("user.home") + File.separator + ".popcorn-time" + File.separator;
    }
}
