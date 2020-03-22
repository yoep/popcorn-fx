package com.github.yoep.popcorn.view.controllers.tv.components;

import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.view.controls.BackgroundImageCover;
import com.github.yoep.popcorn.view.services.ImageService;
import javafx.fxml.FXML;
import javafx.scene.image.Image;
import lombok.extern.slf4j.Slf4j;

import java.io.ByteArrayInputStream;

@Slf4j
public abstract class AbstractDetailsComponent<T extends Media> {
    protected  final ImageService imageService;

    protected T media;

    @FXML
    protected BackgroundImageCover backgroundImage;

    protected AbstractDetailsComponent(ImageService imageService) {
        this.imageService = imageService;
    }

    protected void loadBackgroundImage() {
        backgroundImage.reset();
        imageService.loadFanart(media).whenComplete((bytes, throwable) -> {
            if (throwable == null) {
                bytes.ifPresent(e -> backgroundImage.setBackgroundImage(e));
            } else {
                log.error(throwable.getMessage(), throwable);
            }
        });
    }
}
