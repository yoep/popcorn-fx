package com.github.yoep.player.popcorn.controllers.components;

import com.github.spring.boot.javafx.stereotype.ViewController;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.player.popcorn.controls.StreamInfo;
import com.github.yoep.player.popcorn.controls.StreamInfoCell;
import com.github.yoep.player.popcorn.services.PlaybackService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import javafx.scene.input.MouseEvent;
import lombok.RequiredArgsConstructor;

import java.net.URL;
import java.util.ResourceBundle;

@ViewController
@RequiredArgsConstructor
public class PlayerHeaderComponent implements Initializable {
    private final PlaybackService playbackService;
    private final LocaleText localeText;

    @FXML
    Label title;
    @FXML
    Label quality;
    @FXML
    StreamInfo streamInfo;

    //region Methods

    /**
     * Update the header title.
     *
     * @param title The title to set in the header.
     */
    public void updateTitle(String title) {
        Platform.runLater(() -> this.title.setText(title));
    }

    /**
     * Update the playback quality info.
     *
     * @param quality The current playback quality.
     */
    public void updateQuality(String quality) {
        Platform.runLater(() -> this.quality.setText(quality));
    }

    //endregion

    //region Initializable

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializeStreamInfo();
    }

    private void initializeStreamInfo() {
        streamInfo.setFactory(cell -> new StreamInfoCell(localeText.get("torrent_" + cell)));
        streamInfo.setVisible(false);
    }

    //endregion

    //region Functions

    private void reset() {
        Platform.runLater(() -> {
            title.setText(null);
            quality.setText(null);
            quality.setVisible(false);
            streamInfo.setVisible(false);
        });
    }

    @FXML
    void close(MouseEvent event) {
        event.consume();
        playbackService.stop();
    }

    //endregion
}
