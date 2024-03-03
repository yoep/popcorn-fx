package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.backend.adapters.player.PlayRequest;
import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentService;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentStreamListener;
import com.github.yoep.popcorn.backend.adapters.torrent.model.DownloadStatus;
import com.github.yoep.popcorn.backend.adapters.torrent.state.TorrentStreamState;
import com.github.yoep.popcorn.backend.events.ClosePlayerEvent;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.lib.Handle;
import com.github.yoep.popcorn.backend.player.PlayerChanged;
import com.github.yoep.popcorn.backend.player.PlayerManagerListener;
import com.github.yoep.popcorn.backend.services.AbstractListenerService;
import com.github.yoep.popcorn.ui.view.listeners.PlayerExternalListener;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

import javax.annotation.PostConstruct;
import java.util.Optional;

@Slf4j
@Service
@RequiredArgsConstructor
public class PlayerExternalComponentService extends AbstractListenerService<PlayerExternalListener> {
    static final int TIME_STEP_OFFSET = 10000;

    private final PlayerManagerService playerManagerService;
    private final EventPublisher eventPublisher;
    private final TorrentService torrentService;
    private final TorrentStreamListener streamListener = createStreamListener();

    private long time;
    Handle lastKnownStreamCallback;

    public void togglePlaybackState() {
        playerManagerService.getActivePlayer()
                .ifPresent(this::togglePlaybackStateOnPlayer);
    }

    public void closePlayer() {
        eventPublisher.publishEvent(new ClosePlayerEvent(this, ClosePlayerEvent.Reason.USER));
    }

    public void goBack() {
        playerManagerService.getActivePlayer()
                .ifPresent(e -> e.seek(time - TIME_STEP_OFFSET));
    }

    public void goForward() {
        playerManagerService.getActivePlayer()
                .ifPresent(e -> e.seek(time + TIME_STEP_OFFSET));
    }

    @PostConstruct
    void init() {
        playerManagerService.addListener(new PlayerManagerListener() {
            @Override
            public void activePlayerChanged(PlayerChanged playerChange) {
                // no-op
            }

            @Override
            public void playersChanged() {
                // no-op
            }

            @Override
            public void onPlayerPlaybackChanged(PlayRequest request) {
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
            public void onPlayerStateChanged(PlayerState newState) {
                onStateChanged(newState);
            }
        });
    }

    private void togglePlaybackStateOnPlayer(Player e) {
        if (e.getState() == PlayerState.PAUSED) {
            e.resume();
        } else {
            e.pause();
        }
    }

    private void onPlaybackChanged(PlayRequest request) {
        invokeListeners(e -> e.onRequestChanged(request));

        Optional.ofNullable(lastKnownStreamCallback)
                .ifPresent(torrentService::removeListener);
        lastKnownStreamCallback = request.getStreamHandle()
                .map(e -> torrentService.addListener(e, streamListener))
                .orElse(null);
    }

    private void onDurationChanged(long duration) {
        invokeListeners(e -> e.onDurationChanged(duration));
    }

    private void onTimeChanged(long time) {
        this.time = time;
        invokeListeners(e -> e.onTimeChanged(time));
    }

    private void onStateChanged(PlayerState state) {
        invokeListeners(e -> e.onStateChanged(state));
    }

    private void onDownloadStatus(DownloadStatus status) {
        invokeListeners(e -> e.onDownloadStatus(status));
    }

    private TorrentStreamListener createStreamListener() {
        return new TorrentStreamListener() {
            @Override
            public void onStateChanged(TorrentStreamState newState) {
                // no-op
            }

            @Override
            public void onDownloadStatus(DownloadStatus downloadStatus) {
                PlayerExternalComponentService.this.onDownloadStatus(downloadStatus);
            }
        };
    }
}
