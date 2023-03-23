package com.github.yoep.player.popcorn.controllers.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.player.popcorn.controls.StreamInfo;
import com.github.yoep.player.popcorn.services.PlayerHeaderService;
import com.github.yoep.popcorn.backend.events.ClosePlayerEvent;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import javafx.scene.control.Label;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.Spy;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;

import java.net.URL;
import java.util.ResourceBundle;

import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class DesktopHeaderActionsComponentTest {
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private PlayerHeaderService headerService;
    @Mock
    private LocaleText localeText;
    @Mock
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;
    @InjectMocks
    private DesktopHeaderActionsComponent component;

    @BeforeEach
    void setUp() {
        component.quality = new Label();
        component.streamInfo = new StreamInfo();
    }

    @Test
    void testOnCloseClicked() {
        var event = mock(MouseEvent.class);
        component.initialize(url, resourceBundle);

        component.onCloseClicked(event);

        verify(event).consume();
        verify(eventPublisher).publish(new ClosePlayerEvent(component, ClosePlayerEvent.Reason.USER));
    }

    @Test
    void testOnClosePressed() {
        var event = mock(KeyEvent.class);
        when(event.getCode()).thenReturn(KeyCode.ENTER);
        component.initialize(url, resourceBundle);

        component.onClosePressed(event);

        verify(event).consume();
        verify(eventPublisher).publish(new ClosePlayerEvent(component, ClosePlayerEvent.Reason.USER));
    }
}