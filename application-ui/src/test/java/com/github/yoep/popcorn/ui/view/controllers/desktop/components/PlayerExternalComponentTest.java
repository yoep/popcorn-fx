package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.adapters.platform.PlatformProvider;
import com.github.yoep.popcorn.ui.player.PlayerEventService;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import com.github.yoep.popcorn.ui.view.services.PlayerExternalComponentService;
import javafx.scene.input.MouseEvent;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.mockito.Mockito.verify;


@ExtendWith(MockitoExtension.class)
class PlayerExternalComponentTest {
    @Mock
    private ImageService imageService;
    @Mock
    private PlayerEventService playerEventService;
    @Mock
    private PlatformProvider platformProvider;
    @Mock
    private PlayerExternalComponentService playerExternalService;
    @Mock
    private MouseEvent event;
    @InjectMocks
    private PlayerExternalComponent controller;

    @Test
    void testOnPlayPauseClicked_whenInvoked_shouldTogglePlaybackState() {
        controller.onPlayPauseClicked(event);

        verify(event).consume();
        verify(playerExternalService).togglePlaybackState();
    }

    @Test
    void testOnStopClicked_whenInvoked_shouldCloseThePlayer() {
        controller.onStopClicked(event);

        verify(event).consume();
        verify(playerExternalService).closePlayer();
    }
}