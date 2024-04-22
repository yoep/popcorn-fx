package com.github.yoep.popcorn.ui.view.controllers.common.sections;

import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.adapters.player.embaddable.EmbeddablePlayer;
import com.github.yoep.popcorn.backend.events.ClosePlayerEvent;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.PlayerStartedEvent;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.ui.view.ViewLoader;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.Node;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.Pane;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.ResourceBundle;

@Slf4j

@RequiredArgsConstructor
public class PlayerSectionController implements Initializable {
    static final String EXTERNAL_PLAYER_VIEW = "common/components/player-external.component.fxml";

    private final PlayerManagerService playerManagerService;
    private final ViewLoader viewLoader;
    private final EventPublisher eventPublisher;
    private final ApplicationConfig applicationConfig;

    @FXML
    Pane playerSection;
    @FXML
    Pane playerPlayNextPane;

    Pane externalPlayerPane;

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        loadExternalPlayerPane();
        initializePlayNext();
        eventPublisher.register(PlayerStartedEvent.class, event -> {
            playerManagerService.getActivePlayer().ifPresentOrElse(
                    this::onPlayVideo,
                    () -> log.error("Unable to update player section, no player is active"));
            return event;
        });
    }

    //region Functions

    private void loadExternalPlayerPane() {
        log.trace("Loading the external player pane");
        externalPlayerPane = viewLoader.load(EXTERNAL_PLAYER_VIEW);
    }

    private void initializePlayNext() {
        AnchorPane.setBottomAnchor(playerPlayNextPane, applicationConfig.isTvMode() ? 150d : 50d);
    }

    private void onPlayVideo(Player player) {
        if (player.isEmbeddedPlaybackSupported()) {
            useEmbeddedPlayerPane(player);
        } else {
            useExternalPlayerPane();
        }
    }

    private void useEmbeddedPlayerPane(Player player) {
        if (player instanceof EmbeddablePlayer embeddablePlayer) {
            switchPlayerPane(embeddablePlayer.getEmbeddedPlayer());
        } else {
            log.error("Unable to embed player {}, it supports embedded playback but doesn't implement the EmbeddablePlayer interface", player);
        }
    }

    private void useExternalPlayerPane() {
        switchPlayerPane(externalPlayerPane);
    }

    private void switchPlayerPane(Node pane) {
        anchorPane(pane);

        Platform.runLater(() -> {
            var children = playerSection.getChildren();

            // check if the previous player should be removed
            if (children.size() >= 2) {
                children.remove(0);
            }

            // insert the new pane as first child
            children.add(0, pane);
        });
    }

    private void anchorPane(Node pane) {
        AnchorPane.setTopAnchor(pane, 0.0);
        AnchorPane.setRightAnchor(pane, 0.0);
        AnchorPane.setBottomAnchor(pane, 0.0);
        AnchorPane.setLeftAnchor(pane, 0.0);
    }

    @FXML
    void onPlayerPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.BACK_SPACE || event.getCode() == KeyCode.ESCAPE) {
            event.consume();
            eventPublisher.publish(new ClosePlayerEvent(this, ClosePlayerEvent.Reason.USER));
        }
    }

    //endregion
}
