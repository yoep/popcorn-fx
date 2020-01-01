package com.github.yoep.popcorn.torrent;

import com.github.yoep.popcorn.settings.SettingsService;
import com.github.yoep.popcorn.settings.models.TorrentSettings;
import com.github.yoep.popcorn.torrent.listeners.TorrentListener;
import com.github.yoep.popcorn.torrent.models.TorrentHealth;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.io.FileUtils;
import org.springframework.stereotype.Service;
import org.springframework.util.Assert;

import javax.annotation.PreDestroy;
import java.io.IOException;

@Slf4j
@Service
@RequiredArgsConstructor
public class TorrentService {
    private final TorrentStream torrentStream;
    private final SettingsService settingsService;

    //region Getters

    /**
     * Check if the torrent stream has been initialized.
     * This means that the torrent session has been started and has at least 10 DHT nodes available.
     *
     * @return Returns true if initialize, else false.
     */
    public boolean isInitialized() {
        return torrentStream.isInitialized();
    }

    //endregion

    //region Methods

    /**
     * Register the given listener to the {@link TorrentStream}.
     *
     * @param listener The listener to register.
     */
    public void addListener(TorrentListener listener) {
        Assert.notNull(listener, "listener cannot be null");
        torrentStream.addListener(listener);
    }

    /**
     * Start a new torrent stream for the given torrent url.
     *
     * @param torrentUrl The torrent url to start a new stream for.
     */
    public void startStream(String torrentUrl) {
        Assert.hasText(torrentUrl, "torrentUrl cannot be empty");
        torrentStream.startStream(torrentUrl);
    }

    /**
     * Stop the current torrent stream.
     */
    public void stopStream() {
        torrentStream.stopStream();
    }

    /**
     * Calculate the torrent health for the given seeds/peers.
     *
     * @param seeds The total seeds of the torrent.
     * @param peers The total peers of the torrent.
     * @return Returns the torrent health.
     */
    public TorrentHealth calculateHealth(int seeds, int peers) {
        // first calculate the seed/peer ratio
        var ratio = peers > 0 ? ((float) seeds / peers) : seeds;

        // normalize the data. Convert each to a percentage
        // ratio: Anything above a ratio of 5 is good
        double normalizedRatio = Math.min(ratio / 5 * 100, 100);
        // seeds: Anything above 30 seeds is good
        double normalizedSeeds = Math.min(seeds / 30 * 100, 100);

        // weight the above metrics differently
        // ratio is weighted 60% whilst seeders is 40%
        double weightedRatio = normalizedRatio * 0.6;
        double weightedSeeds = normalizedSeeds * 0.4;
        double weightedTotal = weightedRatio + weightedSeeds;

        int scaledTotal = (int) (weightedTotal * 3 / 100);
        TorrentHealth.Status status;

        switch (scaledTotal) {
            case 0:
                status = TorrentHealth.Status.BAD;
                break;
            case 1:
                status = TorrentHealth.Status.MEDIUM;
                break;
            case 2:
                status = TorrentHealth.Status.GOOD;
                break;
            case 3:
                status = TorrentHealth.Status.EXCELLENT;
                break;
            default:
                status = TorrentHealth.Status.UNKNOWN;
                break;
        }

        return new TorrentHealth(status, ratio, seeds, peers);
    }

    //endregion

    //region Functions

    @PreDestroy
    public void destroy() {
        var settings = getSettings();

        if (settings.isAutoCleaningEnabled() && settings.getDirectory().exists()) {
            try {
                log.info("Cleaning torrent directory {}", settings.getDirectory());
                FileUtils.cleanDirectory(settings.getDirectory());
            } catch (IOException ex) {
                log.error(ex.getMessage(), ex);
            }
        }
    }

    private TorrentSettings getSettings() {
        return settingsService.getSettings().getTorrentSettings();
    }

    //endregion
}
