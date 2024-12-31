package com.github.yoep.popcorn.backend.torrent;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentService;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentStreamListener;
import com.github.yoep.popcorn.backend.lib.Handle;
import lombok.RequiredArgsConstructor;
import lombok.ToString;

@ToString
@RequiredArgsConstructor
public class DefaultTorrentService implements TorrentService {
    private final FxLib fxLib;
    private final PopcornFx instance;
    
    @Override
    public Handle addListener(Handle handle, TorrentStreamListener listener) {
        return null;
    }

    @Override
    public void removeListener(Handle callbackHandle) {

    }

    @Override
    public void cleanup() {
        fxLib.cleanup_torrents_directory(instance);
    }
}
