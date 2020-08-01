package com.github.yoep.popcorn.ui.activities;

import com.github.yoep.torrent.adapter.model.Torrent;
import com.github.yoep.torrent.adapter.model.TorrentStream;

public interface PlayTorrentActivity {
    /**
     * Get the torrent that needs to be played.
     *
     * @return Returns the torrent of the play activity.
     */
    Torrent getTorrent();

    /**
     * Get the torrent stream that is being used for playback.
     *
     * @return Returns the torrent stream of the play activity.
     */
    TorrentStream getTorrentStream();
}
