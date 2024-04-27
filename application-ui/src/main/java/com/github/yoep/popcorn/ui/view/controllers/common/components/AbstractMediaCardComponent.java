package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.providers.models.ShowOverview;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.messages.MediaMessage;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import javafx.scene.control.Tooltip;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
public abstract class AbstractMediaCardComponent extends AbstractCardComponent implements Initializable {
    protected final LocaleText localeText;

    @FXML
    Label title;
    @FXML
    Label year;
    @FXML
    Label seasons;

    protected AbstractMediaCardComponent(Media media, LocaleText localeText, ImageService imageService) {
        super(imageService, media);
        this.localeText = localeText;
    }

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        super.initialize(url, resourceBundle);
        initializeText();
    }

    protected void initializeText() {
        title.setText(media.getTitle());
        year.setText(media.getYear());

        if (media instanceof ShowOverview) {
            var show = (ShowOverview) media;
            var text = localeText.get(MediaMessage.SEASONS, show.getNumberOfSeasons());

            if (show.getNumberOfSeasons() > 1) {
                text += localeText.get(MediaMessage.PLURAL);
            }

            seasons.setText(text);
        }

        Tooltip.install(title, new Tooltip(media.getTitle()));
    }
}
