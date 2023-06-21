package com.github.yoep.popcorn.backend.settings.models;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

import java.io.File;

import static org.junit.jupiter.api.Assertions.assertEquals;

class TorrentSettingsTest {
    @TempDir
    public File workingDir;

    @Test
    void testByValue() {
        var settings = new TorrentSettings();
        settings.directory = workingDir.getAbsolutePath();
        settings.cleaningMode = CleaningMode.ON_SHUTDOWN;
        var expected = new TorrentSettings.ByValue();
        expected.directory = workingDir.getAbsolutePath();
        expected.cleaningMode = CleaningMode.ON_SHUTDOWN;

        var result = new TorrentSettings.ByValue(settings);

        assertEquals(expected, result);
    }
}