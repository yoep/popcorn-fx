package com.github.yoep.popcorn.ui.view.controllers;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.PlayVideoEvent;
import com.github.yoep.popcorn.backend.events.ShowDetailsEvent;
import com.github.yoep.popcorn.backend.media.providers.models.MovieDetails;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.ui.view.services.UrlService;
import javafx.scene.Cursor;
import javafx.scene.Scene;
import javafx.scene.input.*;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.Pane;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.Spy;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.boot.ApplicationArguments;
import org.testfx.framework.junit5.ApplicationExtension;
import org.testfx.util.WaitForAsyncUtils;

import java.net.URL;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class MainControllerTest {
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private ViewLoader viewLoader;
    @Mock
    private ApplicationArguments arguments;
    @Mock
    private UrlService urlService;
    @Mock
    private ApplicationConfig applicationConfig;
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
    void testOnMouseDisabled() throws ExecutionException, InterruptedException, TimeoutException {
        var eventFuture = new CompletableFuture<KeyEvent>();
        var targetNode = new Icon();
        var event = new MouseEvent(MouseEvent.MOUSE_CLICKED, 0, 0, 0, 0, MouseButton.PRIMARY, 1, false, false, false, false, true,
                false, false, false, false, false, new PickResult(targetNode, null, 0, 0, null));
        var scene = new Scene(controller.rootPane);
        targetNode.setOnKeyPressed(eventFuture::complete);
        when(viewLoader.load(isA(String.class))).thenReturn(new Pane(), new Pane(), new Pane(), new Pane());
        when(applicationConfig.isMouseDisabled()).thenReturn(true);

        controller.initialize(url, resourceBundle);
        assertEquals(Cursor.NONE, controller.rootPane.getCursor());
        assertTrue(controller.rootPane.getStyleClass().contains(MainController.MOUSE_DISABLED_STYLE_CLASS));

        controller.rootPane.getChildren().add(targetNode);
        targetNode.requestFocus();
        controller.rootPane.fireEvent(event);
        var eventResult = eventFuture.get(200, TimeUnit.MILLISECONDS);
        assertEquals(KeyCode.ENTER, eventResult.getCode());
    }

    @Test
    void testShowDetailsEvent() throws TimeoutException {
        when(viewLoader.load(isA(String.class))).thenReturn(new Pane(), new Pane(), new Pane(), new Pane());
        controller.initialize(url, resourceBundle);

        eventPublisher.publish(new ShowDetailsEvent<>(this, mock(MovieDetails.class)));

        WaitForAsyncUtils.waitFor(200, TimeUnit.MILLISECONDS, () -> controller.rootPane.getChildren().contains(controller.contentPane));
    }

    @Test
    void testPlayVideoEvent() throws TimeoutException {
        when(viewLoader.load(isA(String.class))).thenReturn(new Pane(), new Pane(), new Pane(), new Pane());
        controller.initialize(url, resourceBundle);

        WaitForAsyncUtils.waitFor(100, TimeUnit.MILLISECONDS, () -> controller.playerPane != null);
        eventPublisher.publish(new PlayVideoEvent(this, "http://localhost", "lorem", false));

        WaitForAsyncUtils.waitFor(200, TimeUnit.MILLISECONDS, () -> controller.rootPane.getChildren().contains(controller.playerPane));
    }
}