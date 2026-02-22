package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.popcorn.listeners.AbstractPlaybackListener;
import com.github.yoep.player.popcorn.listeners.PlaybackListener;
import com.github.yoep.player.popcorn.listeners.PlayerHeaderListener;
import com.github.yoep.popcorn.backend.adapters.torrent.model.DownloadStatus;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Player;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ServerStream;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Stream;
import com.github.yoep.popcorn.backend.services.AbstractListenerService;
import com.github.yoep.popcorn.backend.stream.IStreamServer;
import com.github.yoep.popcorn.backend.stream.StreamListener;
import lombok.extern.slf4j.Slf4j;

import java.util.Objects;
import java.util.Optional;

@Slf4j
public class PlayerHeaderService extends AbstractListenerService<PlayerHeaderListener> {
    private final VideoService videoService;
    private final IStreamServer streamServer;

    private final PlaybackListener listener = createListener();
    private final StreamListener streamListener = createStreamListener();

    private String filename;

    public PlayerHeaderService(VideoService videoService, IStreamServer streamServer) {
        Objects.requireNonNull(videoService, "videoService cannot be null");
        Objects.requireNonNull(streamServer, "streamServer cannot be null");
        this.videoService = videoService;
        this.streamServer = streamServer;
        init();
    }

    private void init() {
        videoService.addListener(listener);
    }

    private void onPlayRequest(Player.PlayRequest request) {
        invokeListeners(e -> e.onTitleChanged(request.getTitle()));
        invokeListeners(e -> e.onCaptionChanged(request.getCaption()));
        invokeListeners(e -> e.onQualityChanged(request.getQuality()));
        invokeListeners(e -> e.onStreamStateChanged(request.hasStream()));

        Optional.ofNullable(this.filename)
                .ifPresent(e -> streamServer.removeListener(e, streamListener));
        Optional.ofNullable(request.getStream())
                .filter(e -> request.hasStream())
                .map(ServerStream::getFilename)
                .ifPresent(filename -> {
                    this.filename = filename;
                    streamServer.addListener(filename, streamListener);
                });
    }

    private void onStreamProgressChanged(Stream.StreamStats stats) {
        invokeListeners(e -> e.onDownloadStatusChanged(new DownloadStatus() {
            @Override
            public float progress() {
                return stats.getProgress();
            }

            @Override
            public long connections() {
                return stats.getConnections();
            }

            @Override
            public int downloadSpeed() {
                return stats.getDownloadSpeed();
            }

            @Override
            public int uploadSpeed() {
                return stats.getUploadSpeed();
            }

            @Override
            public long downloaded() {
                return stats.getDownloaded();
            }

            @Override
            public long totalSize() {
                return stats.getTotalSize();
            }
        }));
    }

    private PlaybackListener createListener() {
        return new AbstractPlaybackListener() {
            @Override
            public void onPlay(Player.PlayRequest request) {
                onPlayRequest(request);
            }
        };
    }

    private StreamListener createStreamListener() {
        return new StreamListener() {
            @Override
            public void onStateChanged(Stream.StreamState state) {
                // no-op
            }

            @Override
            public void onStatsChanged(Stream.StreamStats stats) {
                onStreamProgressChanged(stats);
            }
        };
    }
}
