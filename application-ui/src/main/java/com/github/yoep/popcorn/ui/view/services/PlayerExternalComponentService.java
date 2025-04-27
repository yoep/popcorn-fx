package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentListener;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentService;
import com.github.yoep.popcorn.backend.adapters.torrent.model.DownloadStatus;
import com.github.yoep.popcorn.backend.events.ClosePlayerEvent;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Handle;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Player;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.PlayerManagerEvent;
import com.github.yoep.popcorn.backend.player.PlayerManagerListener;
import com.github.yoep.popcorn.backend.services.AbstractListenerService;
import com.github.yoep.popcorn.ui.view.listeners.PlayerExternalListener;
import lombok.extern.slf4j.Slf4j;

import java.util.Optional;

@Slf4j
public class PlayerExternalComponentService extends AbstractListenerService<PlayerExternalListener> {
    static final int TIME_STEP_OFFSET = 10000;

    private final PlayerManagerService playerManagerService;
    private final EventPublisher eventPublisher;
    private final TorrentService torrentService;
    private final TorrentListener streamListener = createStreamListener();

    private long time;
    private Handle torrentHandle;

    public PlayerExternalComponentService(PlayerManagerService playerManagerService, EventPublisher eventPublisher, TorrentService torrentService) {
        this.playerManagerService = playerManagerService;
        this.eventPublisher = eventPublisher;
        this.torrentService = torrentService;
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
        Optional.ofNullable(this.torrentHandle)
                        .ifPresent(handle -> torrentService.removeListener(handle, streamListener));

        Optional.ofNullable(request.getTorrent())
                .filter(e -> request.hasTorrent())
                .map(Player.PlayRequest.Torrent::getHandle)
                .ifPresent(handle -> {
                    this.torrentHandle = handle;
                    torrentService.addListener(handle, streamListener);
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

    private void onDownloadStatus(DownloadStatus status) {
        invokeListeners(e -> e.onDownloadStatus(status));
    }

    private TorrentListener createStreamListener() {
        return PlayerExternalComponentService.this::onDownloadStatus;
    }
}
