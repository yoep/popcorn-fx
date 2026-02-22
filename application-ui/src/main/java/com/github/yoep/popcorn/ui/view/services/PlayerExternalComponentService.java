package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.adapters.torrent.model.DownloadStatus;
import com.github.yoep.popcorn.backend.events.ClosePlayerEvent;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Player;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.PlayerManagerEvent;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ServerStream;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Stream;
import com.github.yoep.popcorn.backend.player.PlayerManagerListener;
import com.github.yoep.popcorn.backend.services.AbstractListenerService;
import com.github.yoep.popcorn.backend.stream.IStreamServer;
import com.github.yoep.popcorn.backend.stream.StreamListener;
import com.github.yoep.popcorn.ui.view.listeners.PlayerExternalListener;
import lombok.extern.slf4j.Slf4j;

import java.util.Objects;
import java.util.Optional;

@Slf4j
public class PlayerExternalComponentService extends AbstractListenerService<PlayerExternalListener> {
    static final int TIME_STEP_OFFSET = 10000;

    private final PlayerManagerService playerManagerService;
    private final EventPublisher eventPublisher;
    private final IStreamServer streamServer;
    private final StreamListener streamListener = createStreamListener();

    private long time;
    private String filename;

    public PlayerExternalComponentService(PlayerManagerService playerManagerService, EventPublisher eventPublisher, IStreamServer streamServer) {
        Objects.requireNonNull(playerManagerService, "playerManagerService cannot be null");
        Objects.requireNonNull(eventPublisher, "eventPublisher cannot be null");
        Objects.requireNonNull(streamServer, "streamServer cannot be null");
        this.playerManagerService = playerManagerService;
        this.eventPublisher = eventPublisher;
        this.streamServer = streamServer;
        init();
    }

    public void togglePlaybackState() {
        playerManagerService.getActivePlayer().whenComplete((player, throwable) -> {
            if (throwable == null) {
                player.ifPresent(this::togglePlaybackStateOnPlayer);
            } else {
                log.error("Failed to retrieve active player", throwable);
            }
        });

    }

    public void closePlayer() {
        eventPublisher.publishEvent(new ClosePlayerEvent(this, ClosePlayerEvent.Reason.USER));
    }

    public void goBack() {
        playerManagerService.getActivePlayer().thenAccept(player ->
                player.ifPresent(e -> e.seek(time - TIME_STEP_OFFSET)));
    }

    public void goForward() {
        playerManagerService.getActivePlayer().whenComplete((player, throwable) -> {
            if (throwable == null) {
                player.ifPresent(e -> e.seek(time + TIME_STEP_OFFSET));
            } else {
                log.error("Failed to retrieve active player", throwable);
            }
        });
    }

    private void init() {
        playerManagerService.addListener(new PlayerManagerListener() {
            @Override
            public void activePlayerChanged(PlayerManagerEvent.ActivePlayerChanged playerChange) {
                // no-op
            }

            @Override
            public void playersChanged() {
                // no-op
            }

            @Override
            public void onPlayerPlaybackChanged(Player.PlayRequest request) {
                onPlaybackChanged(request);
            }

            @Override
            public void onPlayerTimeChanged(Long newTime) {
                onTimeChanged(newTime);
            }

            @Override
            public void onPlayerDurationChanged(Long newDuration) {
                onDurationChanged(newDuration);
            }

            @Override
            public void onPlayerStateChanged(Player.State newState) {
                onStateChanged(newState);
            }
        });
    }

    private void togglePlaybackStateOnPlayer(com.github.yoep.popcorn.backend.adapters.player.Player e) {
        if (e.getState() == Player.State.PAUSED) {
            e.resume();
        } else {
            e.pause();
        }
    }

    private void onPlaybackChanged(Player.PlayRequest request) {
        invokeListeners(e -> e.onRequestChanged(request));
        Optional.ofNullable(this.filename)
                .ifPresent(filename -> streamServer.removeListener(filename, streamListener));
        Optional.ofNullable(request.getStream())
                .filter(e -> request.hasStream())
                .map(ServerStream::getFilename)
                .ifPresent(filename -> {
                    this.filename = filename;
                    streamServer.addListener(filename, streamListener);
                });
    }

    private void onDurationChanged(long duration) {
        invokeListeners(e -> e.onDurationChanged(duration));
    }

    private void onTimeChanged(long time) {
        this.time = time;
        invokeListeners(e -> e.onTimeChanged(time));
    }

    private void onStateChanged(Player.State state) {
        invokeListeners(e -> e.onStateChanged(state));
    }

    private void onDownloadStatus(Stream.StreamStats stats) {
        invokeListeners(e -> e.onDownloadStatus(new DownloadStatus() {
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

    private StreamListener createStreamListener() {
        return new StreamListener() {
            @Override
            public void onStateChanged(Stream.StreamState state) {
                // no-op
            }

            @Override
            public void onStatsChanged(Stream.StreamStats stats) {
                onDownloadStatus(stats);
            }
        };
    }
}
