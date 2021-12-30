package com.github.yoep.popcorn.backend.settings.models;

import org.junit.jupiter.api.Test;

import java.io.File;

import static org.junit.jupiter.api.Assertions.assertEquals;

class TorrentSettingsTest extends AbstractPropertyTest<TorrentSettings> {
    public TorrentSettingsTest() {
        super(TorrentSettings.class);
    }

    @Test
    void testSetDirectory_whenValueIsDifferent_shouldNotifyPropertyChanged() {
        var newValue = new File("");

        settings.setDirectory(newValue);

        assertEquals(TorrentSettings.DIRECTORY_PROPERTY, invokedPropertyName());
        assertEquals(newValue, invokedNewValue());
    }

    @Test
    void testSetAutoCleaningEnabled_whenValueIsDifferent_shouldNotifyPropertyChanged() {
        var newValue = false;

        settings.setAutoCleaningEnabled(newValue);

        assertEquals(TorrentSettings.AUTO_CLEANING_PROPERTY, invokedPropertyName());
        assertEquals(newValue, invokedNewValue());
    }

    @Test
    void testSetConnectionsLimit_whenValueIsDifferent_shouldNotifyPropertyChanged() {
        var newValue = 100;

        settings.setConnectionsLimit(newValue);

        assertEquals(TorrentSettings.CONNECTIONS_LIMIT_PROPERTY, invokedPropertyName());
        assertEquals(newValue, invokedNewValue());
    }

    @Test
    void testSetDownloadRateLimit_whenValueIsDifferent_shouldNotifyPropertyChanged() {
        var newValue = 200;

        settings.setDownloadRateLimit(newValue);

        assertEquals(TorrentSettings.DOWNLOAD_RATE_PROPERTY, invokedPropertyName());
        assertEquals(newValue, invokedNewValue());
    }

    @Test
    void testSetUploadRateLimit_whenValueIsDifferent_shouldNotifyPropertyChanged() {
        var newValue = 400;

        settings.setUploadRateLimit(newValue);

        assertEquals(TorrentSettings.UPLOAD_RATE_PROPERTY, invokedPropertyName());
        assertEquals(newValue, invokedNewValue());
    }
}