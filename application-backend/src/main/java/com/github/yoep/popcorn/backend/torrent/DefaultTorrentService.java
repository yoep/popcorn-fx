package com.github.yoep.popcorn.backend.torrent;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentListener;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentService;
import com.github.yoep.popcorn.backend.lib.Handle;
import lombok.RequiredArgsConstructor;
import lombok.ToString;

@ToString
@RequiredArgsConstructor
public class DefaultTorrentService implements TorrentService {
    private final FxLib fxLib;
    private final PopcornFx instance;

    @Override
    public void addListener(Handle handle, TorrentListener listener) {
        fxLib.register_torrent_event_callback(instance, handle.nativeHandle(), event -> handleCallback(event, listener));
    }

    @Override
    public void cleanup() {
        fxLib.cleanup_torrents_directory(instance);
    }

    private static void handleCallback(TorrentEventC event, TorrentListener listener) {
        try (event) {
            switch (event.getTag()) {
                case STATE_CHANGED -> {}
                case DOWNLOAD_STATUS -> listener.onDownloadStatus(event.getUnion().getDownloadStatus_body().getStatus());
            }
        }
    }
}
