package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.popcorn.controllers.components.PlayerControlsComponent;
import com.github.yoep.popcorn.ui.view.services.FullscreenService;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Service;

import javax.annotation.PostConstruct;

@Service
@RequiredArgsConstructor
public class FullScreenEventService {
    private final FullscreenService fullscreenService;
    private final PlayerControlsComponent playerControlsComponent;

    @PostConstruct
    void init() {
        initializeListeners();
    }

    private void initializeListeners() {
        fullscreenService.fullscreenProperty().addListener((observableValue, oldValue, newValue) -> {
            playerControlsComponent.updateFullscreenState(newValue);
        });
    }
}
