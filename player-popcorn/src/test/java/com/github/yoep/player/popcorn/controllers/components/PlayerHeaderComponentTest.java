package com.github.yoep.player.popcorn.controllers.components;

import com.github.yoep.player.popcorn.listeners.PlayerHeaderListener;
import com.github.yoep.player.popcorn.services.PlayerHeaderService;
import com.github.yoep.popcorn.ui.view.ViewLoader;
import javafx.scene.control.Label;
import javafx.scene.layout.GridPane;
import javafx.scene.layout.Pane;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;
import org.testfx.util.WaitForAsyncUtils;

import java.net.URL;
import java.util.ResourceBundle;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.*;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class PlayerHeaderComponentTest {
    @Mock
    private PlayerHeaderService playerHeaderService;
    @Mock
    private ViewLoader viewLoader;
    @Mock
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;
    @InjectMocks
    private PlayerHeaderComponent component;

    private final AtomicReference<PlayerHeaderListener> headerListener = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        doAnswer(invocation -> {
            headerListener.set(invocation.getArgument(0, PlayerHeaderListener.class));
            return null;
        }).when(playerHeaderService).addListener(isA(PlayerHeaderListener.class));
        component.playerHeader = new GridPane();
        component.title = new Label();
        component.separator = new Label();
        component.caption = new Label();
    }

    @Test
    void testInitializeMode() throws TimeoutException {
        var actions = new Pane();
        when(viewLoader.load(isA(String.class))).thenReturn(actions);

        component.initialize(url, resourceBundle);

        verify(viewLoader).load(PlayerHeaderComponent.VIEW_HEADER_ACTIONS);
        WaitForAsyncUtils.waitFor(200, TimeUnit.MILLISECONDS, () -> component.playerHeader.getChildren().contains(actions));
    }

    @Test
    void testOnTitleChanged() throws TimeoutException {
        var title = "lorem ipsum";
        when(viewLoader.load(PlayerHeaderComponent.VIEW_HEADER_ACTIONS)).thenReturn(new Pane());
        component.initialize(url, resourceBundle);

        var listener = headerListener.get();
        listener.onTitleChanged(title);

        WaitForAsyncUtils.waitFor(250, TimeUnit.MILLISECONDS, () -> {
            var text = component.title.getText();
            return text != null && !text.isBlank();
        });
        assertEquals(title, component.title.getText());
    }

    @Test
    void testOnCaptionChanged() throws TimeoutException {
        var caption = "foo bar";
        when(viewLoader.load(PlayerHeaderComponent.VIEW_HEADER_ACTIONS)).thenReturn(new Pane());
        component.initialize(url, resourceBundle);

        var listener = headerListener.get();
        listener.onCaptionChanged(caption);

        WaitForAsyncUtils.waitFor(250, TimeUnit.MILLISECONDS, () -> {
            var text = component.caption.getText();
            return text != null && !text.isBlank();
        });
        assertEquals(caption, component.caption.getText());
        assertTrue(component.separator.isVisible());
    }

    @Test
    void testOnCaptionChanged_whenCaptionIsEmpty_shouldHideSeparator() throws TimeoutException {
        var caption = " ";
        when(viewLoader.load(PlayerHeaderComponent.VIEW_HEADER_ACTIONS)).thenReturn(new Pane());
        component.initialize(url, resourceBundle);
        component.caption.setText("Lorem ipsum Dolor");

        var listener = headerListener.get();
        listener.onCaptionChanged(caption);

        WaitForAsyncUtils.waitFor(250, TimeUnit.MILLISECONDS, () -> {
            var text = component.caption.getText();
            return text.isBlank();
        });
        assertEquals(caption, component.caption.getText());
        assertFalse(component.separator.isVisible());
    }
}