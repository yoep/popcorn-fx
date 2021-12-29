package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.popcorn.listeners.PlayerControlsListener;
import com.github.yoep.popcorn.backend.adapters.screen.ScreenService;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

import javax.annotation.PostConstruct;
import java.util.Objects;
import java.util.Queue;
import java.util.concurrent.ConcurrentLinkedQueue;
import java.util.function.Consumer;

@Slf4j
@Service
@RequiredArgsConstructor
public class PlayerControlsService {
    private final ScreenService screenService;

    private final Queue<PlayerControlsListener> listeners = new ConcurrentLinkedQueue<>();

    //region Methods

    public void addListener(PlayerControlsListener listener) {
        Objects.requireNonNull(listener, "listener cannot be null");
        listeners.add(listener);
    }

    public void toggleFullscreen() {
        screenService.toggleFullscreen();
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    void init() {
        screenService.fullscreenProperty().addListener((observableValue, oldValue, newValue) -> invokeListeners(e ->e.onFullscreenStateChanged(newValue)));
    }

    //endregion

    //region Functions

    private void invokeListeners(Consumer<PlayerControlsListener> action) {
        listeners.forEach(action);
    }

    //endregion
}
