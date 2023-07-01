package com.github.yoep.popcorn.backend.adapters.torrent;

import com.github.yoep.popcorn.backend.adapters.torrent.model.Torrent;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentFileInfo;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentHealth;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentInfo;
import com.github.yoep.popcorn.backend.adapters.torrent.state.SessionState;
import javafx.beans.property.ReadOnlyObjectProperty;
import org.springframework.scheduling.annotation.Async;

import java.io.File;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;

/**
 * The {@link TorrentService} manages the {@link Torrent}'s and creation of them.
 * Use this service to resolve magnet url's or start downloading a torrent.
 */
public interface TorrentService {
    /**
     * Get the state of the torrent session.
     *
     * @return Returns the current session state.
     */
    SessionState getSessionState();

    /**
     * Get the session state property of this torrent service.
     *
     * @return Returns the session state property of this service.
     */
    ReadOnlyObjectProperty<SessionState> sessionStateProperty();

    /**
     * Get the error that occurred in the torrent session.
     * The {@link TorrentException} might only be present if the {@link #getSessionState()} is {@link SessionState#ERROR}.
     *
     * @return Returns the torrent session error.
     */
    Optional<TorrentException> getSessionError();

    /**
     * Get the torrent metadata, either by downloading the .torrent or fetching the magnet.
     *
     * @param torrentUrl The URL to the .torrent file or a magnet link.
     * @return Returns the torrent information.
     */
    @Async
    CompletableFuture<TorrentInfo> getTorrentInfo(String torrentUrl);

    /**
     * Get the torrent health for the given torrent url.
     *
     * @param url The torrent url to retrieve the health state of.
     * @param torrentDirectory The directory where the torrent data will be stored.
     * @return Returns the health of the torrent.
     * @throws TorrentException Is thrown when an error occurred during retrieval of the health info.
     */
    @Async
    CompletableFuture<TorrentHealth> getTorrentHealth(String url, File torrentDirectory);

    /**
     * Get the torrent health for the given file info.
     *
     * @param torrentFile The torrent file to return the health of.
     * @param torrentDirectory The directory where the torrent data will be stored.
     * @return Returns the health of the torrent.
     * @throws TorrentException Is thrown when an error occurred during retrieval of the health info.
     */
    @Async
    CompletableFuture<TorrentHealth> getTorrentHealth(TorrentFileInfo torrentFile, File torrentDirectory);

    /**
     * Create a new torrent for the given torrent file.
     *
     * @param torrentFile      The torrent file that needs to be downloaded.
     * @param torrentDirectory The directory where the torrent will be stored.
     * @return Returns the torrent for the given torrent file.
     */
    @Async
    CompletableFuture<Torrent> create(TorrentFileInfo torrentFile, File torrentDirectory);

    /**
     * Create a new torrent for the given torrent file.
     *
     * @param torrentFile       The torrent file that needs to be downloaded.
     * @param torrentDirectory  The directory where the torrent will be stored.
     * @param autoStartDownload Set if the download of the torrent should be started automatically.
     * @return Returns the torrent for the given torrent file.
     */
    @Async
    CompletableFuture<Torrent> create(TorrentFileInfo torrentFile, File torrentDirectory, boolean autoStartDownload);

    /**
     * Remove the given torrent from the session.
     *
     * @param torrent The torrent to remove.
     */
    void remove(Torrent torrent);

    /**
     * Calculate the health of a torrent base on it's seeds and peers.
     *
     * @param seeds The number of seeds.
     * @param peers The number of peers.
     * @return Returns the health of the torrent.
     */
    TorrentHealth calculateHealth(int seeds, int peers);

    /**
     * Clean the torrents directory.
     * This will remove all torrents from the system.
     */
    void cleanup();
}
