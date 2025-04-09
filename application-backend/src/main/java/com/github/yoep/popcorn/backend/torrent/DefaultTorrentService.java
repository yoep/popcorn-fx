package com.github.yoep.popcorn.backend.torrent;

import com.github.yoep.popcorn.backend.adapters.torrent.TorrentListener;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentService;
import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.Handle;
import lombok.ToString;

@ToString
public class DefaultTorrentService implements TorrentService {
    private final FxChannel fxChannel;

    public DefaultTorrentService(FxChannel fxChannel) {
        this.fxChannel = fxChannel;
    }

    @Override
    public void addListener(Handle handle, TorrentListener listener) {
        // TODO
    }

    @Override
    public void cleanup() {
        // TODO
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
