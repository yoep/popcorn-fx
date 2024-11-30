package com.github.yoep.popcorn.backend.adapters.torrent;

import com.github.yoep.popcorn.backend.adapters.torrent.model.Torrent;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentFileInfo;
import com.github.yoep.popcorn.backend.lib.Handle;
import javafx.beans.property.ReadOnlyObjectProperty;

import java.io.File;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;

/**
 * The {@link TorrentService} manages the {@link Torrent}'s and creation of them.
 * Use this service to resolve magnet url's or start downloading a torrent.
 */
public interface TorrentService {
    /**
     * Create a new torrent for the given torrent file.
     *
     * @param torrentFile      The torrent file that needs to be downloaded.
     * @param torrentDirectory The directory where the torrent will be stored.
     * @return Returns the torrent for the given torrent file.
     */
    CompletableFuture<Torrent> create(TorrentFileInfo torrentFile, File torrentDirectory);

    /**
     * Create a new torrent for the given torrent file.
     *
     * @param torrentFile       The torrent file that needs to be downloaded.
     * @param torrentDirectory  The directory where the torrent will be stored.
     * @param autoStartDownload Set if the download of the torrent should be started automatically.
     * @return Returns the torrent for the given torrent file.
     */
    CompletableFuture<Torrent> create(TorrentFileInfo torrentFile, File torrentDirectory, boolean autoStartDownload);

    /**
     * Remove the given torrent from the session.
     *
     * @param torrent The torrent to remove.
     */
    void remove(Torrent torrent);

    Handle addListener(Handle handle, TorrentStreamListener listener);
    
    void removeListener(Handle callbackHandle);

    /**
     * Clean the torrents directory.
     * This will remove all torrents from the system.
     */
    void cleanup();
}
