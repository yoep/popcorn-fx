package com.github.yoep.player.popcorn.controllers.sections;

import com.github.spring.boot.javafx.stereotype.ViewController;
import com.github.yoep.player.popcorn.services.PlaybackService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.scene.Node;
import javafx.scene.control.Label;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.Pane;
import lombok.RequiredArgsConstructor;

@ViewController
@RequiredArgsConstructor
public class PopcornPlayerSectionController {
    private final PlaybackService playbackService;

    @FXML
    Pane videoView;

    //region Methods

    /**
     * Update the video view with the given node.
     *
     * @param view The view node to use.
     */
    public void setVideoView(Node view) {
        videoView.getChildren().setAll(view);
    }

    //endregion

    //region Functions

    @FXML
    void onPlayerClick(MouseEvent event) {
        event.consume();
        playbackService.togglePlayerPlaybackState();
    }

    //endregion
}
