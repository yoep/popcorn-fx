package com.github.yoep.popcorn.ui.view.controllers.common.sections;

import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.events.ClosePlayerEvent;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.PlayerStartedEvent;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.ui.view.ViewLoader;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
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
import java.util.Optional;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class PlayerSectionControllerTest {
    @Mock
    private PlayerManagerService playerManagerService;
    @Mock
    private ViewLoader viewLoader;
    @Mock
    private ApplicationConfig applicationConfig;
    @Spy
    private EventPublisher eventPublisher = new EventPublisher();
    @Mock
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;
    @InjectMocks
    private PlayerSectionController controller;

    private final Pane playNextPane = new Pane();

    @BeforeEach
    void setUp() {
        controller.playerSection = new Pane();
        controller.playerPlayNextPane = new Pane();

        controller.playerSection.getChildren().add(playNextPane);
    }

    @Test
    void testInit_whenInvoked_shouldLoadTheExternalPlayerView() {
        var pane = new Pane();
        when(viewLoader.load(PlayerSectionController.EXTERNAL_PLAYER_VIEW)).thenReturn(pane);

        controller.initialize(url, resourceBundle);

        assertEquals(pane, controller.externalPlayerPane);
    }

    @Test
    void testPlayVideo_whenPlayerDoesNotSupportEmbedding_shouldUseExternalPlayerView() {
        var player = mock(Player.class);
        var externalPlayerPane = new Pane();
        var event = mock(PlayerStartedEvent.class);
        when(viewLoader.load(PlayerSectionController.EXTERNAL_PLAYER_VIEW)).thenReturn(externalPlayerPane);
        when(playerManagerService.getActivePlayer()).thenReturn(CompletableFuture.completedFuture(Optional.of(player)));
        when(player.isEmbeddedPlaybackSupported()).thenReturn(false);
        controller.initialize(url, resourceBundle);

        eventPublisher.publish(event);
        WaitForAsyncUtils.waitForFxEvents(10);

        assertEquals(2, controller.playerSection.getChildren().size(), "Expected a player to have been added");
        assertEquals(externalPlayerPane, controller.playerSection.getChildren().get(0));
    }

    @Test
    void testPlayVideo_whenPlayerSupportsEmbedding_shouldUsePlayerNode() {
        var player = mock(Player.class);
        var playerViewNode = new Pane();
        var externalPlayerPane = new Pane();
        var event = mock(PlayerStartedEvent.class);
        when(viewLoader.load(PlayerSectionController.EXTERNAL_PLAYER_VIEW)).thenReturn(externalPlayerPane);
        when(playerManagerService.getActivePlayer()).thenReturn(CompletableFuture.completedFuture(Optional.of(player)));
        when(player.isEmbeddedPlaybackSupported()).thenReturn(true);
        when(player.getEmbeddedPlayer()).thenReturn(Optional.of(playerViewNode));
        controller.initialize(url, resourceBundle);

        eventPublisher.publish(event);
        WaitForAsyncUtils.waitForFxEvents(10);

        assertEquals(2, controller.playerSection.getChildren().size(), "Expected a player to have been added");
        assertEquals(playerViewNode, controller.playerSection.getChildren().get(0));
    }

    @Test
    void testPlayVideo_whenPlayerViewIsSwitched_shouldRemovePreviousPlayerView() {
        var player1 = mock(Player.class);
        var player2 = mock(Player.class);
        var player1ViewNode = new Pane();
        var player2ViewNode = new Pane();
        var externalPlayerPane = new Pane();
        var event = mock(PlayerStartedEvent.class);
        when(viewLoader.load(PlayerSectionController.EXTERNAL_PLAYER_VIEW)).thenReturn(externalPlayerPane);
        when(playerManagerService.getActivePlayer())
                .thenReturn(CompletableFuture.completedFuture(Optional.of(player1)))
                .thenReturn(CompletableFuture.completedFuture(Optional.of(player2)));
        when(player1.isEmbeddedPlaybackSupported()).thenReturn(true);
        when(player2.isEmbeddedPlaybackSupported()).thenReturn(true);
        when(player1.getEmbeddedPlayer()).thenReturn(Optional.of(player1ViewNode));
        when(player2.getEmbeddedPlayer()).thenReturn(Optional.of(player2ViewNode));
        controller.initialize(url, resourceBundle);

        eventPublisher.publish(event);
        WaitForAsyncUtils.waitForFxEvents(10);
        eventPublisher.publish(event);
        WaitForAsyncUtils.waitForFxEvents(10);

        assertEquals(2, controller.playerSection.getChildren().size(), "Expected the previous player to have been cleared");
        assertEquals(player2ViewNode, controller.playerSection.getChildren().get(0));
    }

    @Test
    void testOnPlayerPressed() {
        var backEvent = mock(KeyEvent.class);
        var escapeEvent = mock(KeyEvent.class);
        when(backEvent.getCode()).thenReturn(KeyCode.BACK_SPACE);
        when(escapeEvent.getCode()).thenReturn(KeyCode.ESCAPE);
        controller.initialize(url, resourceBundle);

        controller.onPlayerPressed(backEvent);
        verify(backEvent).consume();
        verify(eventPublisher).publish(new ClosePlayerEvent(controller, ClosePlayerEvent.Reason.USER));

        controller.onPlayerPressed(escapeEvent);
        verify(escapeEvent).consume();
        verify(eventPublisher, times(2)).publish(new ClosePlayerEvent(controller, ClosePlayerEvent.Reason.USER));
    }
}