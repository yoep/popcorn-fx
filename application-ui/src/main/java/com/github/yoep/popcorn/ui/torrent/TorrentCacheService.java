package com.github.yoep.popcorn.ui.torrent;

import com.github.yoep.popcorn.ui.settings.SettingsService;
import com.github.yoep.popcorn.ui.settings.models.TorrentSettings;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.io.FileUtils;
import org.springframework.stereotype.Service;

import javax.annotation.PreDestroy;
import java.io.File;
import java.io.IOException;

/**
 * The {@link TorrentCacheService} manages the cache which is used by the {@link com.github.yoep.torrent.adapter.TorrentService}.
 */
@Slf4j
@Service
@RequiredArgsConstructor
public class TorrentCacheService {
    private final SettingsService settingsService;

    //region PreDestroy

    @PreDestroy
    void onDestroy() {
        var settings = getSettings();
        var torrentDirectory = getTorrentDirectory();

        if (settings.isAutoCleaningEnabled() && torrentDirectory.exists()) {
            try {
                log.info("Cleaning torrent cache directory {}", torrentDirectory);
                FileUtils.cleanDirectory(torrentDirectory);
            } catch (IOException ex) {
                log.error("Failed to clean cache directory, " + ex.getMessage(), ex);
            }
        }
    }

    //endregion

    //region Functions

    private File getTorrentDirectory() {
        return getSettings().getDirectory();
    }

    private TorrentSettings getSettings() {
        return settingsService.getSettings().getTorrentSettings();
    }

    //endregion
}
