package com.github.yoep.popcorn.ui.view.controllers.desktop.sections;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.ui.events.CloseSettingsEvent;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseButton;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.Pane;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;

import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class SettingsSectionControllerTest {
    @Mock
    private EventPublisher eventPublisher;
    @InjectMocks
    private SettingsSectionController controller;

    @BeforeEach
    void setUp() {
        controller.settings = new Pane();
        controller.settings.setOnKeyPressed(controller::onSettingsPressed);
        controller.settings.setOnMousePressed(controller::onSettingsPressed);
    }

    @Test
    void testOnDetailsPressed_whenKeyEvent_shouldCloseTheDetails() {
        var escEvent = mock(KeyEvent.class);
        when(escEvent.getCode()).thenReturn(KeyCode.ESCAPE);
        var backspaceEvent = mock(KeyEvent.class);
        when(backspaceEvent.getCode()).thenReturn(KeyCode.BACK_SPACE);

        controller.settings.getOnKeyPressed().handle(escEvent);
        verify(escEvent).consume();
        verify(eventPublisher).publishEvent(new CloseSettingsEvent(controller));
        reset(eventPublisher);

        controller.settings.getOnKeyPressed().handle(backspaceEvent);
        verify(backspaceEvent).consume();
        verify(eventPublisher).publishEvent(new CloseSettingsEvent(controller));
    }

    @Test
    void testOnDetailsPressed_whenMouseBackEvent_shouldCloseTheDetails() {
        var event = mock(MouseEvent.class);
        when(event.getButton()).thenReturn(MouseButton.BACK);

        controller.settings.getOnMousePressed().handle(event);

        verify(event).consume();
        verify(eventPublisher).publishEvent(new CloseSettingsEvent(controller));
    }
}