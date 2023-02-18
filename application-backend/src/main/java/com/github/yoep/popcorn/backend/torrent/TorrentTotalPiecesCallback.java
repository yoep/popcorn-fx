package com.github.yoep.popcorn.backend.torrent;

import com.sun.jna.Callback;

interface TorrentTotalPiecesCallback extends Callback {
    int callback();
}
