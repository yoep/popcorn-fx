package com.github.yoep.popcorn.ui.messages;

import com.github.yoep.popcorn.backend.utils.Message;
import lombok.Getter;

@Getter
public enum TorrentMessage implements Message {
    INITIALIZING("torrent_initializing"),
    CONNECTING("torrent_connecting"),
    STARTING("torrent_starting"),
    DOWNLOADING("torrent_downloading"),
    RETRIEVING_SUBTITLES("torrent_retrieving_subtitles"),
    DOWNLOADING_SUBTITLE("torrent_downloading_subtitles"),
    RETRIEVING_METADATA("torrent_retrieving_metadata"),
    READY("torrent_ready"),
    FAILED("torrent_failed"),
    STORE_COLLECTION("torrent_store_collection"),
    REMOVE_COLLECTION("torrent_remove_collection"),
    MAGNET_COPIED("torrent_magnet_copied");

    private final String key;

    TorrentMessage(String key) {
        this.key = key;
    }
}
