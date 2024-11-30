package com.github.yoep.popcorn.backend.torrent;

import com.github.yoep.popcorn.backend.adapters.torrent.TorrentService;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentStreamListener;
import com.github.yoep.popcorn.backend.adapters.torrent.model.Torrent;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentFileInfo;
import com.github.yoep.popcorn.backend.lib.Handle;

import java.io.File;
import java.util.concurrent.CompletableFuture;

public class DefaultTorrentService implements TorrentService {
    @Override
    public CompletableFuture<Torrent> create(TorrentFileInfo torrentFile, File torrentDirectory) {
        return null;
    }

    @Override
    public CompletableFuture<Torrent> create(TorrentFileInfo torrentFile, File torrentDirectory, boolean autoStartDownload) {
        return null;
    }

    @Override
    public void remove(Torrent torrent) {

    }

    @Override
    public Handle addListener(Handle handle, TorrentStreamListener listener) {
        return null;
    }

    @Override
    public void removeListener(Handle callbackHandle) {

    }

    @Override
    public void cleanup() {

    }
}
