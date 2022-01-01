package com.github.yoep.popcorn.backend.adapters.player;

import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentStream;

import javax.validation.constraints.NotNull;

public interface PlayStreamRequest extends PlayRequest {
    /**
     * Get the torrent stream info.
     *
     * @return Returns the torrent stream info.
     */
    @NotNull
    TorrentStream getTorrentStream();
}
