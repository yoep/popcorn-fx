package com.github.yoep.player.popcorn.controllers.components;

import com.github.spring.boot.javafx.stereotype.ViewController;
import com.github.yoep.player.popcorn.services.PlaybackService;
import javafx.fxml.FXML;
import javafx.scene.input.MouseEvent;
import lombok.RequiredArgsConstructor;

@ViewController
@RequiredArgsConstructor
public class PlayerHeaderComponent {
    private final PlaybackService playbackService;

    //region Functions

    @FXML
    void close(MouseEvent event) {
        event.consume();
        playbackService.stop();
    }

    //endregion
}
