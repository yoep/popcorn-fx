package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.yoep.popcorn.backend.adapters.player.PlayRequest;
import com.github.yoep.popcorn.backend.player.PlayerAction;
import com.github.yoep.popcorn.ui.view.controls.BackgroundImageCover;
import com.github.yoep.popcorn.ui.view.controls.ProgressControl;
import com.github.yoep.popcorn.ui.view.listeners.PlayerExternalListener;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import com.github.yoep.popcorn.ui.view.services.PlayerExternalComponentService;
import javafx.scene.control.Label;
import javafx.scene.image.Image;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.core.io.ClassPathResource;
import org.testfx.framework.junit5.ApplicationExtension;
import org.testfx.util.WaitForAsyncUtils;

import java.io.IOException;
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
        controller.captionText = new Label();
        controller.playbackProgress = new ProgressControl();
        controller.infoComponent.progressPercentage = new Label();
    }

    @Test
    void testListener_whenMediaItemIsChanged_shouldLoadBackgroundImage() throws IOException {
        var background = "MyBackgroundUri.jpg";
        var request = mock(PlayRequest.class);
        var holder = new ClassPathResource("posterholder.png");
        when(request.getBackground()).thenReturn(Optional.of(background));
        when(imageService.load(isA(String.class))).thenReturn(CompletableFuture.completedFuture(new Image(holder.getInputStream())));
        controller.init();

        var listener = externalListenerHolder.get();
        listener.onRequestChanged(request);

        verify(imageService).load(background);
    }

    @Test
    void testListener_whenTitleIsChanged_shouldUpdateTitle() throws TimeoutException {
        var title = "Lorem ipsum dolor";
        var request = mock(PlayRequest.class);
        when(request.getTitle()).thenReturn(title);
        controller.init();

        var listener = externalListenerHolder.get();
        listener.onRequestChanged(request);

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