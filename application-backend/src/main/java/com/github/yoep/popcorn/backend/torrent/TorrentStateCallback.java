package com.github.yoep.popcorn.backend.torrent;

import com.github.yoep.popcorn.backend.adapters.torrent.state.TorrentState;
import com.sun.jna.Callback;

interface TorrentStateCallback extends Callback {
    TorrentState callback();
}
