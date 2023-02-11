package com.github.yoep.popcorn.backend.torrent;

import com.sun.jna.Callback;

interface TorrentHasPieceCallback extends Callback {
    byte callback(int piece);
}
