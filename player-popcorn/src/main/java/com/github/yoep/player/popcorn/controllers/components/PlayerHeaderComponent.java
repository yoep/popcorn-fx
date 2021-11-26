package com.github.yoep.player.popcorn.controllers.components;

import com.github.spring.boot.javafx.stereotype.ViewController;
import com.github.yoep.player.popcorn.controls.StreamInfo;
import com.github.yoep.player.popcorn.services.PlaybackService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.scene.control.Label;
import javafx.scene.input.MouseEvent;
import lombok.RequiredArgsConstructor;

@ViewController
@RequiredArgsConstructor
public class PlayerHeaderComponent {
    private final PlaybackService playbackService;

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

    //region Functions

    @FXML
    void close(MouseEvent event) {
        event.consume();
        playbackService.stop();
    }

    //endregion
}
