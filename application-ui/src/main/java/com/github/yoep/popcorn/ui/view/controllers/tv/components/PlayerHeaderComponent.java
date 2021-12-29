package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.yoep.popcorn.backend.events.PlayVideoEvent;
import com.github.yoep.popcorn.ui.events.ClosePlayerEvent;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.scene.control.Label;
import lombok.RequiredArgsConstructor;
import org.springframework.context.event.EventListener;

@RequiredArgsConstructor
public class PlayerHeaderComponent {
    @FXML
    private Label title;

    //region Methods

    @EventListener
    public void onPlayVideo(PlayVideoEvent event) {
        // set the title of the video as it should be always present
        Platform.runLater(() -> {
            this.title.setText(event.getTitle());
        });
    }

    @EventListener(ClosePlayerEvent.class)
    public void onClose() {
        reset();
    }

    //endregion

    //region Functions

    private void reset() {
        Platform.runLater(() -> title.setText(null));
    }

    //endregion
}
