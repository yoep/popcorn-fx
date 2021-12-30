package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.popcorn.listeners.PlaybackListener;
import com.github.yoep.player.popcorn.listeners.PlayerControlsListener;
import com.github.yoep.popcorn.backend.adapters.player.PlayRequest;
import com.github.yoep.popcorn.backend.adapters.screen.ScreenService;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

import javax.annotation.PostConstruct;

@Slf4j
@Service
@RequiredArgsConstructor
public class PlayerControlsService extends AbstractListenerService<PlayerControlsListener> implements PlaybackListener {
    private final ScreenService screenService;

    //region PlaybackListener

    @Override
    public void onPlay(PlayRequest request) {
        invokeListeners(e -> e.onSubtitleStateChanged(request.isSubtitlesEnabled()));
    }

    @Override
    public void onResume() {
        // no-op
    }

    @Override
    public void onPause() {
        // no-op
    }

    @Override
    public void onSeek(long time) {
        // no-op
    }

    @Override
    public void onVolume(int volume) {
        // no-op
    }

    @Override
    public void onStop() {
        // no-op
    }

    //endregion

    //region Methods

    public void toggleFullscreen() {
        screenService.toggleFullscreen();
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    void init() {
        screenService.fullscreenProperty().addListener((observableValue, oldValue, newValue) -> invokeListeners(e -> e.onFullscreenStateChanged(newValue)));
    }

    //endregion
}
