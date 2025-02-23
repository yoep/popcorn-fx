package com.github.yoep.popcorn.backend.adapters.torrent;

import com.github.yoep.popcorn.backend.adapters.torrent.model.DownloadStatus;

public interface TorrentListener {
    void onDownloadStatus(DownloadStatus downloadStatus);
}
