package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.ShowDetailsEvent;
import com.github.yoep.popcorn.backend.media.providers.Media;
import com.github.yoep.popcorn.ui.view.controls.ImageCover;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.scene.layout.Pane;
import lombok.extern.slf4j.Slf4j;

import java.util.Objects;
import java.util.Optional;

@Slf4j
public class TvPosterComponent {
    private final EventPublisher eventPublisher;
    private final ImageService imageService;

    protected Media media;

    @FXML
    Pane posterHolder;
    @FXML
    ImageCover poster;

    public TvPosterComponent(EventPublisher eventPublisher, ImageService imageService) {
        Objects.requireNonNull(eventPublisher, "eventPublisher cannot be null");
        Objects.requireNonNull(imageService, "imageService cannot be null");
        this.eventPublisher = eventPublisher;
        this.imageService = imageService;
        init();
    }

    private void init() {
        eventPublisher.register(ShowDetailsEvent.class, e -> {
            Optional.ofNullable(e.getMedia())
                    .ifPresent(this::onShowDetailsEvent);
            return e;
        });
    }

    void onShowDetailsEvent(Media media) {
        if (Objects.equals(media, this.media))
            return;

        this.media = media;
        updatePoster();
    }

    private void updatePoster() {
        Platform.runLater(() -> poster.setImage(imageService.getPosterPlaceholder(poster.getPrefWidth(), poster.getPrefHeight())));

        imageService.loadPoster(media).whenComplete((image, throwable) -> {
            if (throwable == null) {
                Platform.runLater(() -> image
                        .filter(e -> !e.isError())
                        .ifPresent(poster::setImage));
            } else {
                log.error("Failed to load poster image, {}", throwable.getMessage(), throwable);
            }
        });
    }
}
