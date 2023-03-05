package com.github.yoep.popcorn.backend.settings.models;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

import java.io.File;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class TorrentSettingsTest {
    @TempDir
    public File workingDir;

    @Test
    void testByValue() {
        var settings = new TorrentSettings();
        settings.directory = workingDir.getAbsolutePath();
        settings.autoCleaningEnabled = (byte) 1;
        var expected = new TorrentSettings.ByValue();
        expected.directory = workingDir.getAbsolutePath();
        expected.autoCleaningEnabled = (byte) 1;

        var result = new TorrentSettings.ByValue(settings);

        assertEquals(expected, result);
    }

    @Test
    void testGetAutoCleaningEnabled() {
        var settings = new TorrentSettings();
        settings.autoCleaningEnabled = (byte) 1;

        var result = settings.isAutoCleaningEnabled();

        assertTrue(result);
    }

    @Test
    void testSetAutoCleaningEnabled() {
        var settings = new TorrentSettings();

        settings.setAutoCleaningEnabled(true);

        assertEquals(1, settings.autoCleaningEnabled);
    }
}