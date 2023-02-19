package com.github.yoep.popcorn.ui.torrent;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFxInstance;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentInfo;
import com.github.yoep.popcorn.backend.torrent.collection.StoredTorrent;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

import java.util.List;
import java.util.Objects;

@Slf4j
@Service
@RequiredArgsConstructor
public class TorrentCollectionService {
    /**
     * Check if the given magnet uri has already been added to the torrent collection.
     *
     * @param magnetUri The magnet uri.
     * @return Returns true if the magnet has already been added.
     */
    public boolean isStored(String magnetUri) {
        Objects.requireNonNull(magnetUri, "magnetUri cannot be empty");
        return FxLib.INSTANCE.torrent_collection_is_stored(PopcornFxInstance.INSTANCE.get(), magnetUri) == 1;
    }

    /**
     * Get the current stored torrent collection.
     *
     * @return Returns the stored torrents.
     */
    public List<StoredTorrent> getStoredTorrents() {
        try (var set = FxLib.INSTANCE.torrent_collection_all(PopcornFxInstance.INSTANCE.get())) {
            return set.getMagnets();
        }
    }

    /**
     * Add the given torrent to the torrent collection.
     *
     * @param magnetUri The magnet uri of the torrent.
     * @param torrent   The torrent info to add.
     */
    public void addTorrent(String magnetUri, TorrentInfo torrent) {
        Objects.requireNonNull(torrent, "torrent cannot be null");
        FxLib.INSTANCE.torrent_collection_add(PopcornFxInstance.INSTANCE.get(), torrent.getName(), magnetUri);
    }

    /**
     * Remove the given magnet uri from the torrent collection.
     *
     * @param magnetUri The magnet uri to remove.
     */
    public void removeTorrent(String magnetUri) {
        Objects.requireNonNull(magnetUri, "magnetUri cannot be null");
        FxLib.INSTANCE.torrent_collection_remove(PopcornFxInstance.INSTANCE.get(), magnetUri);
    }
}
