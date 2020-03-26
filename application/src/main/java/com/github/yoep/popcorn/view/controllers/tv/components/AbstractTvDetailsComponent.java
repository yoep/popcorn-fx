package com.github.yoep.popcorn.view.controllers.tv.components;

import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.view.controllers.common.components.AbstractDetailsComponent;
import com.github.yoep.popcorn.view.controls.BackgroundImageCover;
import com.github.yoep.popcorn.view.services.ImageService;
import javafx.fxml.FXML;
import lombok.extern.slf4j.Slf4j;

@Slf4j
public abstract class AbstractTvDetailsComponent<T extends Media> extends AbstractDetailsComponent<T> {
    protected AbstractTvDetailsComponent(ImageService imageService) {
        super(imageService);
    }


}
