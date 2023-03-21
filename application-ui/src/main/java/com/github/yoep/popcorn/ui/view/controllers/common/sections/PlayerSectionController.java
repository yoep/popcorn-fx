package com.github.yoep.popcorn.ui.view.controllers.common.sections;

import com.github.spring.boot.javafx.stereotype.ViewController;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.adapters.player.embaddable.EmbeddablePlayer;
import com.github.yoep.popcorn.backend.events.ClosePlayerEvent;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.PlayVideoEvent;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.scene.Node;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.Pane;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import javax.annotation.PostConstruct;

@Slf4j
@ViewController
@RequiredArgsConstructor
public class PlayerSectionController {
    static final String EXTERNAL_PLAYER_VIEW = "common/components/player-external.component.fxml";

    private final PlayerManagerService playerManagerService;
    private final ViewLoader viewLoader;
    private final EventPublisher eventPublisher;

    @FXML
    Pane playerSectionPane;

    Pane externalPlayerPane;

    //region PostConstruct

    @PostConstruct
    void init() {
        loadExternalPlayerPane();
        eventPublisher.register(PlayVideoEvent.class, event -> {
            playerManagerService.getActivePlayer().ifPresentOrElse(
                    this::onPlayVideo,
                    () -> log.error("Unable to update player section, no player is active"));
            return event;
        });
    }

    //endregion

    //region Functions

    private void loadExternalPlayerPane() {
        log.trace("Loading the external player pane");
        externalPlayerPane = viewLoader.load(EXTERNAL_PLAYER_VIEW);
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
            var children = playerSectionPane.getChildren();

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
