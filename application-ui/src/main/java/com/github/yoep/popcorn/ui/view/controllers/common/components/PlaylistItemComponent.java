package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Playlist;
import com.github.yoep.popcorn.ui.view.controls.ImageCover;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import javafx.scene.layout.Pane;
import lombok.EqualsAndHashCode;
import lombok.RequiredArgsConstructor;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.Optional;
import java.util.ResourceBundle;

@Slf4j
@ToString
@EqualsAndHashCode
@RequiredArgsConstructor
public class PlaylistItemComponent implements Initializable {
    static final String ACTIVE_CLASS = "active";

    private final Playlist.Item item;
    private final ImageService imageService;

    @FXML
    Pane itemPane;
    @FXML
    ImageCover thumbnail;
    @FXML
    Label title;
    @FXML
    Label caption;

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        imageService.getPosterPlaceholder().whenComplete((poster, throwable) -> {
            if (throwable == null) {
                Platform.runLater(() -> thumbnail.setImage(poster));
            } else {
                log.error("Failed to load the post placeholder", throwable);
            }
        });
        Optional.ofNullable(item.getThumb()).ifPresent(this::loadThumbnail);
        title.setText(item.getTitle());
        caption.setText(item.getCaption());
    }

    public void setActive(boolean isActive) {
        if (isActive) {
            itemPane.getStyleClass().add(ACTIVE_CLASS);
        } else {
            itemPane.getStyleClass().removeIf(e -> e.equals(ACTIVE_CLASS));
        }
    }

    private void loadThumbnail(String url) {
        imageService.load(url).whenComplete((image, throwable) -> {
            if (throwable == null) {
                Platform.runLater(() -> thumbnail.setImage(image));
            } else {
                log.warn("Failed to load thumbnail, {}", throwable.getMessage(), throwable);
            }
        });
    }
}
