package com.github.yoep.popcorn.ui.view.controllers.common.sections;

import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.adapters.player.embaddable.EmbeddablePlayer;
import com.github.yoep.popcorn.backend.events.ClosePlayerEvent;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.PlayVideoEvent;
import com.github.yoep.popcorn.backend.settings.OptionsService;
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

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class PlayerSectionControllerTest {
    @Mock
    private PlayerManagerService playerManagerService;
    @Mock
    private ViewLoader viewLoader;
    @Mock
    private OptionsService optionsService;
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
        controller.playerSectionPane = new Pane();
        controller.playerPlayNextPane = new Pane();

        controller.playerSectionPane.getChildren().add(playNextPane);
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
        var event = mock(PlayVideoEvent.class);
        when(viewLoader.load(PlayerSectionController.EXTERNAL_PLAYER_VIEW)).thenReturn(externalPlayerPane);
        when(playerManagerService.getActivePlayer()).thenReturn(Optional.of(player));
        when(player.isEmbeddedPlaybackSupported()).thenReturn(false);
        controller.initialize(url, resourceBundle);

        eventPublisher.publish(event);
        WaitForAsyncUtils.waitForFxEvents(10);

        assertEquals(2, controller.playerSectionPane.getChildren().size(), "Expected a player to have been added");
        assertEquals(externalPlayerPane, controller.playerSectionPane.getChildren().get(0));
    }

    @Test
    void testPlayVideo_whenPlayerSupportsEmbedding_shouldUsePlayerNode() {
        var player = mock(EmbeddablePlayer.class);
        var playerViewNode = new Pane();
        var externalPlayerPane = new Pane();
        var event = mock(PlayVideoEvent.class);
        when(viewLoader.load(PlayerSectionController.EXTERNAL_PLAYER_VIEW)).thenReturn(externalPlayerPane);
        when(playerManagerService.getActivePlayer()).thenReturn(Optional.of(player));
        when(player.isEmbeddedPlaybackSupported()).thenReturn(true);
        when(player.getEmbeddedPlayer()).thenReturn(playerViewNode);
        controller.initialize(url, resourceBundle);

        eventPublisher.publish(event);
        WaitForAsyncUtils.waitForFxEvents(10);

        assertEquals(2, controller.playerSectionPane.getChildren().size(), "Expected a player to have been added");
        assertEquals(playerViewNode, controller.playerSectionPane.getChildren().get(0));
    }

    @Test
    void testPlayVideo_whenPlayerViewIsSwitched_shouldRemovePreviousPlayerView() {
        var player1 = mock(EmbeddablePlayer.class);
        var player2 = mock(EmbeddablePlayer.class);
        var player1ViewNode = new Pane();
        var player2ViewNode = new Pane();
        var externalPlayerPane = new Pane();
        var event = mock(PlayVideoEvent.class);
        when(viewLoader.load(PlayerSectionController.EXTERNAL_PLAYER_VIEW)).thenReturn(externalPlayerPane);
        when(playerManagerService.getActivePlayer()).thenReturn(Optional.of(player1), Optional.of(player2));
        when(player1.isEmbeddedPlaybackSupported()).thenReturn(true);
        when(player2.isEmbeddedPlaybackSupported()).thenReturn(true);
        when(player1.getEmbeddedPlayer()).thenReturn(player1ViewNode);
        when(player2.getEmbeddedPlayer()).thenReturn(player2ViewNode);
        controller.initialize(url, resourceBundle);

        eventPublisher.publish(event);
        WaitForAsyncUtils.waitForFxEvents(10);
        eventPublisher.publish(event);
        WaitForAsyncUtils.waitForFxEvents(10);

        assertEquals(2, controller.playerSectionPane.getChildren().size(), "Expected the previous player to have been cleared");
        assertEquals(player2ViewNode, controller.playerSectionPane.getChildren().get(0));
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