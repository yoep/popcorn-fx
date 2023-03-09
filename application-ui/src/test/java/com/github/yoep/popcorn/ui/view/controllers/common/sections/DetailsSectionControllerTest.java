package com.github.yoep.popcorn.ui.view.controllers.common.sections;

import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.ui.events.CloseDetailsEvent;
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
import org.mockito.Spy;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.core.task.TaskExecutor;
import org.testfx.framework.junit5.ApplicationExtension;

import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class DetailsSectionControllerTest {
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private ViewLoader viewLoader;
    @Mock
    private TaskExecutor taskExecutor;
    @InjectMocks
    private DetailsSectionController controller;

    @BeforeEach
    void setUp() {
        controller.detailPane = new Pane();
        controller.detailPane.setOnKeyPressed(controller::onDetailsPressed);
        controller.detailPane.setOnMousePressed(controller::onDetailsPressed);
    }

    @Test
    void testOnDetailsPressed_whenKeyEvent_shouldCloseTheDetails() {
        var escEvent = mock(KeyEvent.class);
        when(escEvent.getCode()).thenReturn(KeyCode.ESCAPE);
        var backspaceEvent = mock(KeyEvent.class);
        when(backspaceEvent.getCode()).thenReturn(KeyCode.BACK_SPACE);
        controller.init();

        controller.detailPane.getOnKeyPressed().handle(escEvent);
        verify(escEvent).consume();
        verify(eventPublisher).publishEvent(new CloseDetailsEvent(controller));
        reset(eventPublisher);

        controller.detailPane.getOnKeyPressed().handle(backspaceEvent);
        verify(backspaceEvent).consume();
        verify(eventPublisher).publishEvent(new CloseDetailsEvent(controller));
    }

    @Test
    void testOnDetailsPressed_whenMouseBackEvent_shouldCloseTheDetails() {
        var event = mock(MouseEvent.class);
        when(event.getButton()).thenReturn(MouseButton.BACK);
        controller.init();

        controller.detailPane.getOnMousePressed().handle(event);

        verify(event).consume();
        verify(eventPublisher).publishEvent(new CloseDetailsEvent(controller));
    }
}