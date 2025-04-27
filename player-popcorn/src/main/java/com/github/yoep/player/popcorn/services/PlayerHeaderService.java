package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.popcorn.listeners.AbstractPlaybackListener;
import com.github.yoep.player.popcorn.listeners.PlaybackListener;
import com.github.yoep.player.popcorn.listeners.PlayerHeaderListener;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentListener;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentService;
import com.github.yoep.popcorn.backend.adapters.torrent.model.DownloadStatus;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Handle;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Player;
import com.github.yoep.popcorn.backend.services.AbstractListenerService;
import lombok.extern.slf4j.Slf4j;

import java.util.Objects;
import java.util.Optional;

@Slf4j
public class PlayerHeaderService extends AbstractListenerService<PlayerHeaderListener> {
    private final VideoService videoService;
    private final TorrentService torrentService;

    private final PlaybackListener listener = createListener();
    private final TorrentListener torrentListener = createTorrentStreamListener();

    private Handle handle;

    public PlayerHeaderService(VideoService videoService, TorrentService torrentService) {
        Objects.requireNonNull(videoService, "videoService cannot be null");
        Objects.requireNonNull(torrentService, "torrentService cannot be null");
        this.videoService = videoService;
        this.torrentService = torrentService;
        init();
    }

    private void init() {
        videoService.addListener(listener);
    }

    private void onPlayRequest(Player.PlayRequest request) {
        invokeListeners(e -> e.onTitleChanged(request.getTitle()));
        invokeListeners(e -> e.onCaptionChanged(request.getCaption()));
        invokeListeners(e -> e.onQualityChanged(request.getQuality()));
        invokeListeners(e -> e.onStreamStateChanged(request.hasTorrent()));

        Optional.ofNullable(this.handle)
                .ifPresent(handle -> torrentService.removeListener(handle, torrentListener));
        Optional.ofNullable(request.getTorrent())
                .filter(e -> request.hasTorrent())
                .map(Player.PlayRequest.Torrent::getHandle)
                .ifPresent(handle -> {
                    this.handle = handle;
                    torrentService.addListener(handle, torrentListener);
                });
    }

    private void onStreamProgressChanged(DownloadStatus status) {
        invokeListeners(e -> e.onDownloadStatusChanged(status));
    }

    private PlaybackListener createListener() {
        return new AbstractPlaybackListener() {
            @Override
            public void onPlay(Player.PlayRequest request) {
                onPlayRequest(request);
            }
        };
    }

    private TorrentListener createTorrentStreamListener() {
        return this::onStreamProgressChanged;
    }
}
