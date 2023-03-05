package com.github.yoep.popcorn.ui.view.controllers;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.ui.view.services.MaximizeService;
import javafx.scene.input.MouseButton;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.Pane;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.context.ApplicationEventPublisher;
import org.testfx.framework.junit5.ApplicationExtension;

import java.net.URL;
import java.util.ResourceBundle;

import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class ContentSectionControllerTest {
    @Mock
    private ViewLoader viewLoader;
    @Mock
    private LocaleText localeText;
    @Mock
    private ApplicationEventPublisher eventPublisher;
    @Mock
    private MaximizeService maximizeService;
    @Mock
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;
    @InjectMocks
    private ContentSectionController controller;

    @BeforeEach
    void setUp() {
        lenient().when(viewLoader.load(isA(String.class))).thenReturn(new Pane());
        controller.contentPane = new Pane();
    }

    @Test
    void testMouseDoubleClicked_shouldToggleTheMaximizeState() {
        var event = mock(MouseEvent.class);
        when(event.getSceneY()).thenReturn(30.0);
        when(event.getButton()).thenReturn(MouseButton.PRIMARY);
        when(event.getClickCount()).thenReturn(2);
        controller.initialize(url, resourceBundle);

        var action = controller.contentPane.getOnMouseClicked();
        action.handle(event);

        verify(maximizeService).setMaximized(true);
    }
}