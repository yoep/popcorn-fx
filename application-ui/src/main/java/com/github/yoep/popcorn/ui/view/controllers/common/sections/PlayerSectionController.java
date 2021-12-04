package com.github.yoep.popcorn.ui.view.controllers.common.sections;

import com.github.spring.boot.javafx.stereotype.ViewController;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.player.adapter.Player;
import com.github.yoep.player.adapter.PlayerManagerService;
import com.github.yoep.player.adapter.embaddable.EmbeddablePlayer;
import com.github.yoep.popcorn.ui.events.PlayVideoEvent;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.Node;
import javafx.scene.layout.Pane;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.event.EventListener;

import javax.annotation.PostConstruct;
import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
@ViewController
@RequiredArgsConstructor
public class PlayerSectionController implements Initializable {
    private static final String EXTERNAL_PLAYER_VIEW = "components/player-external.component.fxml";

    private final PlayerManagerService playerManagerService;
    private final ViewLoader viewLoader;

    @FXML
    private Pane playerContentPane;

    private Pane externalPlayerPane;

    //region Methods

    @EventListener(PlayVideoEvent.class)
    public void onPlayVideo() {
        playerManagerService.getActivePlayer().ifPresentOrElse(
                this::onPlayVideo,
                () -> log.error("Unable to update player section, no player is active"));
    }

    //endregion

    //region Initializable

    @Override
    public void initialize(URL location, ResourceBundle resources) {

    }

    //endregion

    //region PostConstruct

    @PostConstruct
    void init() {
        loadExternalPlayerPane();
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
        if (!(player instanceof EmbeddablePlayer)) {
            log.error("Unable to embed player {}, it supports embedded playback but doesn't implement the EmbeddablePlayer interface", player);
            return;
        }

        var embeddablePlayer = (EmbeddablePlayer) player;
        switchPlayerPane(embeddablePlayer.getEmbeddedPlayer());
    }

    private void useExternalPlayerPane() {
        switchPlayerPane(externalPlayerPane);
    }

    private void switchPlayerPane(Node pane) {
        Platform.runLater(() -> playerContentPane.getChildren().setAll(pane));
    }

    //endregion
}
