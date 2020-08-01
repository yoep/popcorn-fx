package com.github.yoep.torrent.frostwire;

import com.frostwire.jlibtorrent.Priority;
import com.frostwire.jlibtorrent.TorrentHandle;
import com.github.yoep.torrent.adapter.TorrentException;
import com.github.yoep.torrent.adapter.TorrentService;
import com.github.yoep.torrent.adapter.model.Torrent;
import com.github.yoep.torrent.adapter.model.TorrentFileInfo;
import com.github.yoep.torrent.adapter.model.TorrentHealth;
import com.github.yoep.torrent.adapter.model.TorrentInfo;
import com.github.yoep.torrent.adapter.state.SessionState;
import com.github.yoep.torrent.adapter.state.TorrentHealthState;
import com.github.yoep.torrent.frostwire.listeners.TorrentCreationListener;
import com.github.yoep.torrent.frostwire.model.FrostTorrent;
import com.github.yoep.torrent.frostwire.model.FrostTorrentHealth;
import com.github.yoep.torrent.frostwire.wrappers.TorrentInfoWrapper;
import javafx.beans.property.ReadOnlyObjectProperty;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.util.Assert;

import java.io.File;
import java.util.Arrays;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.atomic.AtomicReference;

@Slf4j
@RequiredArgsConstructor
public class TorrentServiceImpl implements TorrentService {
    private final TorrentSessionManager sessionManager;
    private final TorrentResolverService torrentResolverService;

    //region Getters

    @Override
    public SessionState getSessionState() {
        return sessionManager.getState();
    }

    @Override
    public ReadOnlyObjectProperty<SessionState> sessionStateProperty() {
        return sessionManager.stateProperty();
    }

    @Override
    public Optional<TorrentException> getSessionError() {
        return Optional.ofNullable(sessionManager.getError());
    }

    //endregion

    //region Methods

    @Override
    public CompletableFuture<TorrentInfo> getTorrentInfo(String torrentUrl) {
        return CompletableFuture.completedFuture(torrentResolverService.resolveUrl(torrentUrl));
    }

    @Override
    public CompletableFuture<Torrent> create(TorrentFileInfo torrentFile, File torrentDirectory) {
        return create(torrentFile, torrentDirectory, false);
    }

    @Override
    public CompletableFuture<Torrent> create(TorrentFileInfo torrentFile, File torrentDirectory, boolean autoStartDownload) {
        Assert.notNull(torrentFile, "torrentFile cannot be null");
        Assert.notNull(torrentDirectory, "torrentDirectory cannot be null");
        log.debug("Creating new torrent for {} in {}", torrentFile.getFilename(), torrentDirectory.getAbsolutePath());
        var session = sessionManager.getSession();
        var torrentInfo = (TorrentInfoWrapper) torrentFile.getTorrentInfo();
        var priorities = new Priority[torrentInfo.getTotalFiles()];
        var handle = new AtomicReference<TorrentHandle>();

        // by default, ignore all files
        // this should prevent the torrent from starting to download immediately
        Arrays.fill(priorities, Priority.IGNORE);

        // create a new torrent creation listener
        new TorrentCreationListener(session)
                .onComplete(torrentHandle -> {
                    synchronized (this) {
                        handle.set(torrentHandle);
                        notifyAll();
                    }
                })
                .register();

        // start the creation of the torrent by downloading it
        session.download(torrentInfo.getNative(), torrentDirectory, null, priorities, null);

        // pause this thread and wait for the torrent to be created
        synchronized (this) {
            try {
                wait();
            } catch (InterruptedException ex) {
                log.error("Torrent creation monitor unexpectedly quit", ex);
            }
        }

        // create a new torrent from the handle
        var torrent = new FrostTorrent(handle.get(), torrentFile.getFileIndex(), autoStartDownload);

        // register the torrent in the session
        session.addListener(torrent);

        return CompletableFuture.completedFuture(torrent);
    }

    @Override
    public void remove(Torrent torrent) {
        Assert.notNull(torrent, "torrent cannot be null");
        var session = sessionManager.getSession();

        // pause the torrent download
        torrent.pause();

        // check if the torrent can be removed from the session
        if (torrent instanceof FrostTorrent) {
            var frostTorrent = (FrostTorrent) torrent;

            session.removeListener(frostTorrent);
            session.remove(frostTorrent.getHandle());
        } else {
            throw new TorrentException("Invalid torrent, torrent is not a frost torrent type");
        }
    }

    @Override
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
        TorrentHealthState healthState;

        switch (scaledTotal) {
            case 0:
                healthState = TorrentHealthState.BAD;
                break;
            case 1:
                healthState = TorrentHealthState.MEDIUM;
                break;
            case 2:
                healthState = TorrentHealthState.GOOD;
                break;
            case 3:
                healthState = TorrentHealthState.EXCELLENT;
                break;
            default:
                healthState = TorrentHealthState.UNKNOWN;
                break;
        }

        return new FrostTorrentHealth(healthState, ratio, seeds, peers);
    }

    //endregion

    //region Functions


    //endregion
}
