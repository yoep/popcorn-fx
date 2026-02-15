package com.github.yoep.popcorn.ui.view.controllers;

import com.github.yoep.popcorn.backend.adapters.platform.PlatformProvider;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.PlayerStartedEvent;
import com.github.yoep.popcorn.backend.events.ShowDetailsEvent;
import com.github.yoep.popcorn.backend.media.MovieDetails;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.ui.ApplicationArgs;
import com.github.yoep.popcorn.ui.font.controls.Icon;
import com.github.yoep.popcorn.ui.view.ViewLoader;
import com.github.yoep.popcorn.ui.view.services.UrlService;
import javafx.application.Platform;
import javafx.scene.Cursor;
import javafx.scene.Scene;
import javafx.scene.control.Button;
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
import org.testfx.framework.junit5.ApplicationExtension;
import org.testfx.util.WaitForAsyncUtils;

import java.net.URL;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class MainControllerTest {
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private ApplicationArgs applicationArgs;
    @Mock
    private ViewLoader viewLoader;
    @Mock
    private UrlService urlService;
    @Mock
    private ApplicationConfig applicationConfig;
    @Mock
    private PlatformProvider platformProvider;
    @Mock
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;
    @InjectMocks
    private MainController controller;

    @BeforeEach
    void setUp() {
        lenient().when(applicationArgs.args()).thenReturn(new String[0]);
        controller.root = new AnchorPane();
    }

    @Test
    void testOnMouseDisabledMouseEvent() throws TimeoutException {
        var eventFuture = new CompletableFuture<KeyEvent>();
        var targetNode = new Icon();
        var event = new MouseEvent(MouseEvent.MOUSE_CLICKED, 0, 0, 0, 0, MouseButton.PRIMARY, 1, false, false, false, false, true,
                false, false, false, false, false, new PickResult(targetNode, null, 0, 0, null));
        var scene = new Scene(controller.root);
        targetNode.setOnKeyPressed(eventFuture::complete);
        when(viewLoader.load(isA(String.class))).thenReturn(new Pane(), new Pane(), new Pane(), new Pane());
        when(applicationConfig.isMouseDisabled()).thenReturn(true);

        controller.initialize(url, resourceBundle);
        WaitForAsyncUtils.waitFor(200, TimeUnit.MILLISECONDS, () -> controller.loaderPane != null);
        assertEquals(Cursor.NONE, controller.root.getCursor());
        assertTrue(controller.root.getStyleClass().contains(MainController.MOUSE_DISABLED_STYLE_CLASS));

        controller.root.getChildren().add(targetNode);
        targetNode.requestFocus();
        controller.root.fireEvent(event);
        // TODO: fix this test, it doesn't work within github actions
        //        var eventResult = eventFuture.get(200, TimeUnit.MILLISECONDS);
        //        assertEquals(KeyCode.ENTER, eventResult.getCode());
    }

    @Test
    void testOnMouseDisabledKeyEvent() throws TimeoutException {
        var eventFuture = new CompletableFuture<KeyEvent>();
        var targetNode = new Button();
        var event = new KeyEvent(this, targetNode, KeyEvent.KEY_PRESSED, "", "", KeyCode.UNDEFINED, false, false, false, false);
        var scene = new Scene(controller.root);
        targetNode.setOnKeyPressed(eventFuture::complete);
        when(viewLoader.load(isA(String.class))).thenReturn(new Pane(), new Pane(), new Pane(), new Pane());
        when(applicationConfig.isMouseDisabled()).thenReturn(true);

        controller.initialize(url, resourceBundle);
        WaitForAsyncUtils.waitFor(200, TimeUnit.MILLISECONDS, () -> controller.loaderPane != null);

        controller.root.getChildren().add(targetNode);
        targetNode.requestFocus();
        controller.root.fireEvent(event);
    }

    @Test
    void testShowDetailsEvent() throws TimeoutException {
        when(viewLoader.load(isA(String.class))).thenReturn(new Pane(), new Pane(), new Pane(), new Pane());
        controller.initialize(url, resourceBundle);

        eventPublisher.publish(new ShowDetailsEvent<>(this, mock(MovieDetails.class)));

        WaitForAsyncUtils.waitFor(200, TimeUnit.MILLISECONDS, () -> controller.root.getChildren().contains(controller.contentPane));
    }

    @Test
    void testPlayVideoEvent() throws TimeoutException {
        when(viewLoader.load(isA(String.class))).thenReturn(new Pane(), new Pane(), new Pane(), new Pane());
        controller.initialize(url, resourceBundle);

        WaitForAsyncUtils.waitFor(100, TimeUnit.MILLISECONDS, () -> controller.playerPane != null);
        eventPublisher.publish(PlayerStartedEvent.builder()
                .source(this)
                .build());

        WaitForAsyncUtils.waitFor(200, TimeUnit.MILLISECONDS, () -> controller.root.getChildren().contains(controller.playerPane));
    }

    @Test
    void testClipboardPasteEvent() {
        var magnetUrl = "magnet:?loremDolorEsta";
        var event = new KeyEvent(KeyEvent.KEY_PRESSED, "v", "v", KeyCode.V, false, !isMacOs(), false, isMacOs());
        var clipboardContent = new ClipboardContent();
        clipboardContent.putUrl(magnetUrl);
        when(viewLoader.load(isA(String.class))).thenReturn(new Pane(), new Pane(), new Pane(), new Pane());
        controller.initialize(url, resourceBundle);

        // execute everything on a FX thread
        Platform.runLater(() -> {
            Clipboard.getSystemClipboard().setContent(clipboardContent);
            controller.onKeyPressed(event);
        });
        WaitForAsyncUtils.waitForFxEvents();

        verify(urlService, timeout(250)).process(magnetUrl);
    }

    private static boolean isMacOs() {
        return System.getProperty("os.name").toLowerCase().contains("mac");
    }
}