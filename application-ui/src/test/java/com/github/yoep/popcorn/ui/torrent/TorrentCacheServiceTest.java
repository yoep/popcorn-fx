package com.github.yoep.popcorn.ui.torrent;

import com.github.yoep.popcorn.ui.settings.SettingsService;
import com.github.yoep.popcorn.ui.settings.models.ApplicationSettings;
import com.github.yoep.popcorn.ui.settings.models.TorrentSettings;
import org.apache.commons.io.FileUtils;
import org.junit.After;
import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import java.io.File;
import java.io.IOException;
import java.nio.charset.Charset;
import java.nio.file.Files;

import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.class)
public class TorrentCacheServiceTest {
    @Mock
    private SettingsService settingsService;
    @Mock
    private ApplicationSettings applicationSettings;
    @Mock
    private TorrentSettings torrentSettings;
    @InjectMocks
    private TorrentCacheService torrentCacheService;

    private File tmpDir;
    private File tmpFile;

    @Before
    public void setup() throws IOException {
        createCacheDirectory();
    }

    @After
    public void tearDown() throws IOException {
        FileUtils.deleteDirectory(tmpDir);
    }

    @Test
    public void testOnDestroy_whenCleanCacheIsDisabled_shouldNotCleanTheCacheDirectory() {
        when(settingsService.getSettings()).thenReturn(applicationSettings);
        when(applicationSettings.getTorrentSettings()).thenReturn(torrentSettings);
        when(torrentSettings.isAutoCleaningEnabled()).thenReturn(false);

        torrentCacheService.onDestroy();

        assertTrue(tmpFile.exists());
    }

    @Test
    public void testOnDestroy_whenCleanCacheIsEnabled_shouldCleanTheCacheDirectory() {
        when(settingsService.getSettings()).thenReturn(applicationSettings);
        when(applicationSettings.getTorrentSettings()).thenReturn(torrentSettings);
        when(torrentSettings.isAutoCleaningEnabled()).thenReturn(true);
        when(torrentSettings.getDirectory()).thenReturn(tmpDir);

        torrentCacheService.onDestroy();

        assertFalse(tmpFile.exists());
    }

    private void createCacheDirectory() throws IOException {
        tmpDir = Files.createTempDirectory("popcorn-test-").toFile();
        tmpFile = new File(tmpDir.getAbsolutePath() + "/cache-test.txt");

        FileUtils.writeStringToFile(tmpFile, "lorem ipsum dolor", Charset.defaultCharset());
    }
}
