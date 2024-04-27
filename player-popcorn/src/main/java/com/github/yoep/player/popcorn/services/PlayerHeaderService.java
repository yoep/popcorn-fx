package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.popcorn.listeners.AbstractPlaybackListener;
import com.github.yoep.player.popcorn.listeners.PlaybackListener;
import com.github.yoep.player.popcorn.listeners.PlayerHeaderListener;
import com.github.yoep.popcorn.backend.adapters.player.PlayRequest;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentService;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentStreamListener;
import com.github.yoep.popcorn.backend.adapters.torrent.model.DownloadStatus;
import com.github.yoep.popcorn.backend.adapters.torrent.state.TorrentStreamState;
import com.github.yoep.popcorn.backend.lib.Handle;
import com.github.yoep.popcorn.backend.services.AbstractListenerService;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import javax.annotation.PostConstruct;
import java.util.Optional;

@Slf4j

@RequiredArgsConstructor
public class PlayerHeaderService extends AbstractListenerService<PlayerHeaderListener> {
    private final VideoService videoService;
    private final TorrentService torrentService;

    private final PlaybackListener listener = createListener();
    private final TorrentStreamListener torrentListener = createTorrentStreamListener();

    private Handle lastKnownCallbackHandle;

    @PostConstruct
    void init() {
        videoService.addListener(listener);
    }

    private void onPlayRequest(PlayRequest request) {
        invokeListeners(e -> e.onTitleChanged(request.getTitle()));
        invokeListeners(e -> e.onCaptionChanged(request.getCaption().orElse(null)));
        invokeListeners(e -> e.onQualityChanged(request.getQuality().orElse(null)));
        invokeListeners(e -> e.onStreamStateChanged(request.getStreamHandle().isPresent()));

        Optional.ofNullable(lastKnownCallbackHandle)
                .ifPresent(torrentService::removeListener);

        lastKnownCallbackHandle = request.getStreamHandle()
                .map(e -> torrentService.addListener(e, torrentListener))
                .orElse(null);
    }

    private void onStreamProgressChanged(DownloadStatus status) {
        invokeListeners(e -> e.onDownloadStatusChanged(status));
    }

    private PlaybackListener createListener() {
        return new AbstractPlaybackListener() {
            @Override
            public void onPlay(PlayRequest request) {
                onPlayRequest(request);
            }
        };
    }

    private TorrentStreamListener createTorrentStreamListener() {
        return new TorrentStreamListener() {
            @Override
            public void onStateChanged(TorrentStreamState newState) {
                // no-op
            }

            @Override
            public void onDownloadStatus(DownloadStatus downloadStatus) {
                onStreamProgressChanged(downloadStatus);
            }
        };
    }
}
