package com.github.yoep.popcorn.ui.torrent;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentInfo;
import com.github.yoep.popcorn.backend.settings.SettingsDefaults;
import com.github.yoep.popcorn.ui.torrent.models.StoredTorrent;
import com.github.yoep.popcorn.ui.torrent.models.TorrentCollection;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.io.FileUtils;
import org.springframework.stereotype.Service;
import org.springframework.util.Assert;

import java.io.File;
import java.io.IOException;
import java.nio.charset.Charset;
import java.util.List;
import java.util.Objects;

@Slf4j
@Service
@RequiredArgsConstructor
public class TorrentCollectionService {
    private static final String NAME = "torrent-collection.json";

    private final ObjectMapper objectMapper;

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
        File file = getFile();

        if (file.exists()) {
            try {
                log.debug("Loading torrent collection from {}", file.getAbsolutePath());
                return objectMapper.readValue(file, TorrentCollection.class);
            } catch (IOException ex) {
                log.error("Failed to load torrent collection with error " + ex.getMessage(), ex);
            }
        }

        return new TorrentCollection();
    }

    private void save(TorrentCollection collection) {
        File file = getFile();

        try {
            log.debug("Saving torrent collection to {}", file.getAbsolutePath());
            FileUtils.writeStringToFile(file, objectMapper.writeValueAsString(collection), Charset.defaultCharset());
        } catch (IOException ex) {
            log.error("Failed to save torrent collection with error " + ex.getMessage(), ex);
        }
    }

    private File getFile() {
        return new File(SettingsDefaults.APP_DIR + NAME);
    }
}
