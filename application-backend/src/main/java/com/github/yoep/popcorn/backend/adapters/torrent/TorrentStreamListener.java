package com.github.yoep.popcorn.backend.adapters.torrent;

import com.github.yoep.popcorn.backend.adapters.torrent.model.DownloadStatus;
import com.github.yoep.popcorn.backend.adapters.torrent.state.TorrentStreamState;

public interface TorrentStreamListener {
    void onStateChanged(TorrentStreamState newState);

    void onDownloadStatus(DownloadStatus downloadStatus);
}
