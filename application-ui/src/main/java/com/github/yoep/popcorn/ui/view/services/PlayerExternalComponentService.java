package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.adapters.player.listeners.PlayerListener;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import com.github.yoep.popcorn.backend.events.ClosePlayerEvent;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.loader.*;
import com.github.yoep.popcorn.backend.player.PlayerEventService;
import com.github.yoep.popcorn.backend.services.AbstractListenerService;
import com.github.yoep.popcorn.ui.view.listeners.PlayerExternalListener;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

import javax.annotation.PostConstruct;

@Slf4j
@Service
@RequiredArgsConstructor
public class PlayerExternalComponentService extends AbstractListenerService<PlayerExternalListener> {
    static final int TIME_STEP_OFFSET = 10000;

    private final PlayerManagerService playerManagerService;
    private final PlayerEventService playerEventService;
    private final EventPublisher eventPublisher;
    private final LoaderService loaderService;

    private final PlayerListener playerListener = createListener();
    private long time;

    public void togglePlaybackState() {
        playerManagerService.getActivePlayer()
                .ifPresent(this::togglePlaybackStateOnPlayer);
    }

    public void closePlayer() {
        playerManagerService.getActivePlayer()
                .ifPresent(Player::stop);
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
        playerEventService.addListener(playerListener);
        loaderService.addListener(new LoaderListener() {
            @Override
            public void onLoadingStarted(LoadingStartedEventC loadingStartedEvent) {
                invokeListeners(e -> e.onTitleChanged(loadingStartedEvent.getTitle()));
            }

            @Override
            public void onStateChanged(LoaderState newState) {
                // no-op
            }

            @Override
            public void onProgressChanged(LoadingProgress progress) {
                invokeListeners(e -> e.onDownloadStatus(progress));
            }

            @Override
            public void onError(LoadingErrorC error) {
                // no-op
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

    private PlayerListener createListener() {
        return new PlayerListener() {
            @Override
            public void onDurationChanged(long newDuration) {
                PlayerExternalComponentService.this.onDurationChanged(newDuration);
            }

            @Override
            public void onTimeChanged(long newTime) {
                PlayerExternalComponentService.this.onTimeChanged(newTime);
            }

            @Override
            public void onStateChanged(PlayerState newState) {
                PlayerExternalComponentService.this.onStateChanged(newState);
            }

            @Override
            public void onVolumeChanged(int volume) {
                // no-op
            }
        };
    }
}
