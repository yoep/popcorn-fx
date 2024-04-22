package com.github.yoep.player.popcorn.controllers.components;

import com.github.yoep.player.popcorn.listeners.PlayerHeaderListener;
import com.github.yoep.player.popcorn.services.PlayerHeaderService;
import com.github.yoep.popcorn.backend.adapters.torrent.model.DownloadStatus;
import com.github.yoep.popcorn.ui.view.ViewLoader;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.geometry.HPos;
import javafx.scene.control.Label;
import javafx.scene.layout.GridPane;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
@RequiredArgsConstructor
public class PlayerHeaderComponent implements Initializable {
    static final String VIEW_HEADER_ACTIONS = "components/header-actions.component.fxml";

    private final PlayerHeaderService playerHeaderService;
    private final ViewLoader viewLoader;

    @FXML
    GridPane playerHeader;
    @FXML
    Label title;
    @FXML
    Label separator;
    @FXML
    Label caption;

    //region Initializable

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializeMode();
        initializeListeners();
    }

    private void initializeMode() {
        var actions = viewLoader.load(VIEW_HEADER_ACTIONS);
        GridPane.setHalignment(actions, HPos.RIGHT);
        playerHeader.add(actions, 2, 0);
    }

    private void initializeListeners() {
        playerHeaderService.addListener(new PlayerHeaderListener() {
            @Override
            public void onTitleChanged(String title) {
                Platform.runLater(() -> PlayerHeaderComponent.this.title.setText(title));
            }

            @Override
            public void onCaptionChanged(String caption) {
                PlayerHeaderComponent.this.onCaptionChanged(caption);
            }

            @Override
            public void onQualityChanged(String quality) {
                // no-op
            }

            @Override
            public void onStreamStateChanged(boolean isStreaming) {
                // no-op
            }

            @Override
            public void onDownloadStatusChanged(DownloadStatus progress) {
                // no-op
            }
        });
    }

    private void onCaptionChanged(String caption) {
        Platform.runLater(() -> {
            PlayerHeaderComponent.this.separator.setVisible(caption != null);
            PlayerHeaderComponent.this.caption.setText(caption);
        });
    }

    //endregion
}
