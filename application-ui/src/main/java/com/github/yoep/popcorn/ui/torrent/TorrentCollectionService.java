package com.github.yoep.popcorn.ui.torrent;

import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentInfo;
import com.github.yoep.popcorn.backend.storage.StorageService;
import com.github.yoep.popcorn.ui.torrent.models.StoredTorrent;
import com.github.yoep.popcorn.ui.torrent.models.TorrentCollection;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;
import org.springframework.util.Assert;

import java.util.List;
import java.util.Objects;

@Slf4j
@Service
@RequiredArgsConstructor
public class TorrentCollectionService {
    private static final String STORAGE_NAME = "torrent-collection.json";

    private final StorageService storageService;

    /**
     * Check if the given magnet uri has already been added to the torrent collection.
     *
     * @param magnetUri The magnet uri.
     * @return Returns true if the magnet has already been added.
     */
    public boolean isStored(String magnetUri) {
        Assert.hasText(magnetUri, "magnetUri cannot be empty");
        return loadCollection().getTorrents().stream()
                .anyMatch(e -> Objects.equals(e.getMagnetUri(), magnetUri));
    }

    /**
     * Get the current stored torrent collection.
     *
     * @return Returns the stored torrents.
     */
    public List<StoredTorrent> getStoredTorrents() {
        return loadCollection().getTorrents();
    }

    /**
     * Add the given torrent to the torrent collection.
     *
     * @param magnetUri The magnet uri of the torrent.
     * @param torrent   The torrent info to add.
     */
    public void addTorrent(String magnetUri, TorrentInfo torrent) {
        Assert.notNull(torrent, "torrent cannot be null");
        TorrentCollection collection = loadCollection();

        // check if the torrent has already been stored
        // if so, ignore this action
        if (isStored(magnetUri))
            return;

        collection.getTorrents().add(StoredTorrent.builder()
                .magnetUri(magnetUri)
                .name(torrent.getName())
                .build());

        save(collection);
    }

    /**
     * Remove the given magnet uri from the torrent collection.
     *
     * @param magnetUri The magnet uri to remove.
     */
    public void removeTorrent(String magnetUri) {
        Assert.hasText(magnetUri, "magnetUri cannot be null");
        TorrentCollection collection = loadCollection();

        collection.getTorrents().removeIf(e -> Objects.equals(e.getMagnetUri(), magnetUri));

        save(collection);
    }

    private TorrentCollection loadCollection() {
        log.debug("Loading torrent collection from storage");
        return storageService.read(STORAGE_NAME, TorrentCollection.class)
                .orElse(new TorrentCollection());
    }

    private void save(TorrentCollection collection) {
        log.debug("Saving torrent collection to storage");
        storageService.store(STORAGE_NAME, collection);
    }
}
