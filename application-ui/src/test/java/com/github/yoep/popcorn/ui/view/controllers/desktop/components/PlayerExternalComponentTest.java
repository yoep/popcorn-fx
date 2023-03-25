package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.player.PlayerAction;
import com.github.yoep.popcorn.ui.view.controls.BackgroundImageCover;
import com.github.yoep.popcorn.ui.view.listeners.PlayerExternalListener;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import com.github.yoep.popcorn.ui.view.services.PlayerExternalComponentService;
import javafx.scene.control.Label;
import javafx.scene.control.ProgressBar;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;
import org.testfx.util.WaitForAsyncUtils;

import java.util.Optional;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;
import java.util.concurrent.atomic.AtomicReference;

import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;


@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class PlayerExternalComponentTest {
    @Mock
    private ImageService imageService;
    @Mock
    private PlayerExternalComponentService playerExternalService;
    @Mock
    private MouseEvent event;
    @InjectMocks
    private PlayerExternalComponent controller;

    private final AtomicReference<PlayerExternalListener> externalListenerHolder = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        lenient().doAnswer(invocation -> {
            externalListenerHolder.set(invocation.getArgument(0, PlayerExternalListener.class));
            return null;
        }).when(playerExternalService).addListener(isA(PlayerExternalListener.class));

        controller.backgroundImage = new BackgroundImageCover();
        controller.titleText = new Label();
        controller.progressPercentage = new Label();
        controller.playbackProgress = new ProgressBar();
    }

    @Test
    void testListener_whenMediaItemIsChanged_shouldLoadBackgroundImage() {
        var media = mock(Media.class);
        when(imageService.loadFanart(isA(Media.class))).thenReturn(CompletableFuture.completedFuture(Optional.empty()));
        controller.init();

        var listener = externalListenerHolder.get();
        listener.onMediaChanged(media);

        verify(imageService).loadFanart(media);
    }

    @Test
    void testListener_whenTitleIsChanged_shouldUpdateTitle() throws TimeoutException {
        var title = "Lorem ipsum dolor";
        controller.init();

        var listener = externalListenerHolder.get();
        listener.onTitleChanged(title);

        WaitForAsyncUtils.waitFor(200, TimeUnit.MILLISECONDS, () -> controller.titleText.getText().equals(title));
    }

    @Test
    void testOnPlayPauseClicked_whenInvoked_shouldTogglePlaybackState() {
        controller.onPlayPauseClicked(event);

        verify(event).consume();
        verify(playerExternalService).togglePlaybackState();
    }

    @Test
    void testOnStopClicked_whenInvoked_shouldCloseThePlayer() {
        controller.onStopClicked(event);

        verify(event).consume();
        verify(playerExternalService).closePlayer();
    }

    @Test
    void testOnGoBack_whenInvoked_shouldGoBackInThePlayer() {
        controller.onGoBackClicked(event);

        verify(event).consume();
        verify(playerExternalService).goBack();
    }

    @Test
    void testOnGoForward_whenInvoked_shouldGoForwardInThePlayer() {
        controller.onGoForwardClicked(event);

        verify(event).consume();
        verify(playerExternalService).goForward();
    }

    @Test
    void testOnPaneKeyReleased_whenPauseKeyIsPressed_shouldToggleThePlaybackState() {
        var event = mock(KeyEvent.class);
        when(event.getCode()).thenReturn(PlayerAction.TOGGLE_PLAYBACK_STATE.getKeys()[0]);

        controller.onPaneKeyReleased(event);

        verify(event).consume();
        verify(playerExternalService).togglePlaybackState();
    }

    @Test
    void testOnPaneKeyReleased_whenForwardKeyIsPressed_shouldForwardThePlayer() {
        var event = mock(KeyEvent.class);
        when(event.getCode()).thenReturn(PlayerAction.FORWARD.getKeys()[0]);

        controller.onPaneKeyReleased(event);

        verify(event).consume();
        verify(playerExternalService).goForward();
    }

    @Test
    void testOnPaneKeyReleased_whenReverseKeyIsPressed_shouldReverseThePlayer() {
        var event = mock(KeyEvent.class);
        when(event.getCode()).thenReturn(PlayerAction.REVERSE.getKeys()[0]);

        controller.onPaneKeyReleased(event);

        verify(event).consume();
        verify(playerExternalService).goBack();
    }
}