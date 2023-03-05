package com.github.yoep.popcorn.backend.settings.models;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

import java.io.File;

import static org.junit.jupiter.api.Assertions.assertEquals;

class SubtitleSettingsTest {
    @TempDir
    public File workingDir;

    @Test
    void testSetAutoCleaningEnabled() {
        var settings = new SubtitleSettings();

        settings.setAutoCleaningEnabled(true);

        assertEquals(1, settings.autoCleaningEnabled);
    }

    @Test
    void testByValue() {
        var settings = new SubtitleSettings();
        settings.directory = workingDir.getAbsolutePath();
        settings.autoCleaningEnabled = (byte) 1;

        var result = new SubtitleSettings.ByValue(settings);

        assertEquals(workingDir.getAbsolutePath(), result.directory);
        assertEquals(1, result.autoCleaningEnabled);
    }
}