package com.github.yoep.popcorn.ui.torrent;

import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentInfo;
import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.torrent.collection.StoredTorrent;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.util.List;
import java.util.Objects;

@Slf4j
public class TorrentCollectionService {
    private final FxChannel fxChannel;

    public TorrentCollectionService(FxChannel fxChannel) {
        Objects.requireNonNull(fxChannel, "fxChannel cannot be null");
        this.fxChannel = fxChannel;
    }

    /**
     * Check if the given magnet uri has already been added to the torrent collection.
     *
     * @param magnetUri The magnet uri.
     * @return Returns true if the magnet has already been added.
     */
    public boolean isStored(String magnetUri) {
        Objects.requireNonNull(magnetUri, "magnetUri cannot be empty");
//        return fxLib.torrent_collection_is_stored(instance, magnetUri) == 1;
        return false;
    }

    /**
     * Get the current stored torrent collection.
     *
     * @return Returns the stored torrents.
     */
    public List<StoredTorrent> getStoredTorrents() {
//        try (var set = fxLib.torrent_collection_all(instance)) {
//            return set.getMagnets();
//        }
        return null;
    }

    /**
     * Add the given torrent to the torrent collection.
     *
     * @param torrent The torrent info to add.
     */
    public void addTorrent(TorrentInfo torrent) {
        Objects.requireNonNull(torrent, "torrent cannot be null");
//        fxLib.torrent_collection_add(instance, torrent.getName(), torrent.getMagnetUri());
    }

    /**
     * Remove the given magnet uri from the torrent collection.
     *
     * @param magnetUri The magnet uri to remove.
     */
    public void removeTorrent(String magnetUri) {
        Objects.requireNonNull(magnetUri, "magnetUri cannot be null");
//        fxLib.torrent_collection_remove(instance, magnetUri);
    }
}
