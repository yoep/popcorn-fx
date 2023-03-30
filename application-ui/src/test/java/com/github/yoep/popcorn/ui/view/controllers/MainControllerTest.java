package com.github.yoep.popcorn.ui.view.controllers;

import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.ui.view.services.UrlService;
import javafx.scene.Cursor;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.Pane;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.boot.ApplicationArguments;
import org.springframework.core.task.TaskExecutor;
import org.testfx.framework.junit5.ApplicationExtension;

import java.net.URL;
import java.util.ResourceBundle;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.when;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class MainControllerTest {
    @Mock
    private EventPublisher eventPublisher;
    @Mock
    private ViewLoader viewLoader;
    @Mock
    private ApplicationArguments arguments;
    @Mock
    private UrlService urlService;
    @Mock
    private ApplicationConfig applicationConfig;
    @Mock
    private TaskExecutor taskExecutor;
    @Mock
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;
    @InjectMocks
    private MainController controller;

    @BeforeEach
    void setUp() {
        controller.rootPane = new AnchorPane();
    }

    @Test
    void testOnMouseDisabled() {
        when(viewLoader.load(isA(String.class))).thenReturn(new Pane(), new Pane(), new Pane(), new Pane());
        when(applicationConfig.isMouseDisabled()).thenReturn(true);

        controller.initialize(url, resourceBundle);

        assertEquals(Cursor.NONE, controller.rootPane.getCursor());
    }
}