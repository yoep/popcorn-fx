package com.github.yoep.player.popcorn.controllers.components;

import com.github.yoep.player.popcorn.services.PlayerControlsService;
import com.github.yoep.popcorn.backend.events.ClosePlayerEvent;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.Spy;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;

import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class TvPlayerControlsComponentTest {
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private PlayerControlsService playerControlsService;
    @InjectMocks
    private TvPlayerControlsComponent component;

    @Test
    void testOnPlayClicked() {
        var event = mock(MouseEvent.class);

        component.onPlayClicked(event);

        verify(event).consume();
        verify(playerControlsService).togglePlayerPlaybackState();
    }

    @Test
    void testOnPlayPressed() {
        var event = mock(KeyEvent.class);
        when(event.getCode()).thenReturn(KeyCode.ENTER);

        component.onPlayPressed(event);

        verify(event).consume();
        verify(playerControlsService).togglePlayerPlaybackState();
    }

    @Test
    void testOnStopClicked() {
        var event = mock(MouseEvent.class);

        component.onStopClicked(event);

        verify(event).consume();
        verify(eventPublisher).publish(new ClosePlayerEvent(component, ClosePlayerEvent.Reason.USER));
    }

    @Test
    void testOnStopPressed() {
        var event = mock(KeyEvent.class);
        when(event.getCode()).thenReturn(KeyCode.ENTER);

        component.onStopPressed(event);

        verify(event).consume();
        verify(eventPublisher).publish(new ClosePlayerEvent(component, ClosePlayerEvent.Reason.USER));
    }
}