package com.github.yoep.popcorn.ui.view.controllers.common.sections;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.ui.events.CloseDetailsEvent;
import com.github.yoep.popcorn.ui.view.ViewLoader;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseButton;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.Pane;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.Spy;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;

import java.util.concurrent.ExecutorService;

import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class DetailsSectionControllerTest {
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private ViewLoader viewLoader;
    @Mock
    private ExecutorService executorService;
    @Mock
    private ApplicationConfig applicationConfig;

    @BeforeEach
    void setUp() {
        lenient().doAnswer(invocation -> {
            var runnable = invocation.getArgument(0, Runnable.class);
            runnable.run();
            return null;
        }).when(executorService).execute(any(Runnable.class));
    }

    @Test
    void testOnDetailsPressed_whenKeyEvent_shouldCloseTheDetails() {
        var escEvent = mock(KeyEvent.class);
        when(escEvent.getCode()).thenReturn(KeyCode.ESCAPE);
        var backspaceEvent = mock(KeyEvent.class);
        when(backspaceEvent.getCode()).thenReturn(KeyCode.BACK_SPACE);
        when(viewLoader.load(isA(String.class))).thenReturn(new Pane());
        var controller = createController();

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
        when(viewLoader.load(isA(String.class))).thenReturn(new Pane());
        var controller = createController();

        controller.detailPane.getOnMousePressed().handle(event);

        verify(event).consume();
        verify(eventPublisher).publishEvent(new CloseDetailsEvent(controller));
    }

    private DetailsSectionController createController() {
        var controller = new DetailsSectionController(eventPublisher, viewLoader, executorService, applicationConfig);
        controller.detailPane = new Pane();
        controller.detailPane.setOnKeyPressed(controller::onDetailsPressed);
        controller.detailPane.setOnMousePressed(controller::onDetailsPressed);
        return controller;
    }
}