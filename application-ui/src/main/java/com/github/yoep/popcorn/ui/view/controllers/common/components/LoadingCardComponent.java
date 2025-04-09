package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.fxml.Initializable;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
public class LoadingCardComponent extends AbstractCardComponent implements Initializable {
    public LoadingCardComponent(ImageService imageService) {
        super(imageService);
    }

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializePoster();
    }

    private void initializePoster() {
        imageService.getPosterPlaceholder(POSTER_WIDTH, POSTER_HEIGHT).whenComplete((image, throwable) -> {
            if (throwable == null) {
                setBackgroundImage(image, false);
            } else {
                log.error("Failed to load poster placeholder", throwable);
            }
        });
    }
}
