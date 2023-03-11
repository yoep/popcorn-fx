package com.github.yoep.popcorn.ui.view.controllers.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.media.providers.models.Episode;
import com.github.yoep.popcorn.ui.messages.DetailsMessage;
import com.github.yoep.popcorn.ui.view.controls.ImageCover;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import com.github.yoep.popcorn.ui.view.services.ShowHelperService;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
public class EpisodeComponent implements Initializable {
    private final Episode media;
    private final LocaleText localeText;
    private final ImageService imageService;

    @FXML
    ImageCover episodeArt;
    @FXML
    Label title;
    @FXML
    Label airDate;
    @FXML
    Label synopsis;

    public EpisodeComponent(Episode media, LocaleText localeText, ImageService imageService) {
        this.media = media;
        this.localeText = localeText;
        this.imageService = imageService;
    }

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        episodeArt.setImage(imageService.getArtHolder());
        title.setText(media.getTitle());
        airDate.setText(localeText.get(DetailsMessage.AIR_DATE, ShowHelperService.AIRED_DATE_PATTERN.format(media.getAirDate())));
        synopsis.setText(media.getSynopsis());
    }
}
