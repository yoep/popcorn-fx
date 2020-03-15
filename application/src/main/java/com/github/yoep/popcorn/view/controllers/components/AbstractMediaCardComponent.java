package com.github.yoep.popcorn.view.controllers.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.media.providers.models.Images;
import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.media.providers.models.Show;
import com.github.yoep.popcorn.messages.MediaMessage;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import javafx.scene.control.Tooltip;
import javafx.scene.image.Image;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.Optional;
import java.util.ResourceBundle;

@Slf4j
public abstract class AbstractMediaCardComponent extends AbstractCardComponent implements Initializable {
    protected final Media media;
    protected final LocaleText localeText;

    @FXML
    protected Label title;
    @FXML
    protected Label year;
    @FXML
    protected Label seasons;

    protected AbstractMediaCardComponent(Media media, LocaleText localeText) {
        this.media = media;
        this.localeText = localeText;
    }

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeImage();
        initializeText();
    }

    protected void initializeImage() {
        var imageLoadingThread = new Thread(() -> {
            try {
                // use the post holder image as a default
                var image = new Image(getPosterHolderResource().getInputStream(), POSTER_WIDTH, POSTER_HEIGHT, true, true);
                setBackgroundImage(image, false);
            } catch (Exception ex) {
                log.error(ex.getMessage(), ex);
            }

            // try to load the actual image
            Optional.ofNullable(media.getImages())
                    .map(Images::getPoster)
                    .filter(e -> !e.equalsIgnoreCase("n/a"))
                    .ifPresent(mediaImage -> {
                        try {
                            setBackgroundImage(new Image(mediaImage, POSTER_WIDTH, POSTER_HEIGHT, true, true), true);
                        } catch (Exception ex) {
                            log.error(ex.getMessage(), ex);
                        }
                    });
        }, "MediaCardComponent.ImageLoader");

        // run this on a separate thread for easier UI loading
        imageLoadingThread.start();
    }

    protected void initializeText() {
        title.setText(media.getTitle());
        year.setText(media.getYear());

        if (media instanceof Show) {
            Show show = (Show) media;
            String text = localeText.get(MediaMessage.SEASONS, show.getNumberOfSeasons());

            if (show.getNumberOfSeasons() > 1) {
                text += localeText.get(MediaMessage.PLURAL);
            }

            seasons.setText(text);
        }

        Tooltip.install(title, new Tooltip(media.getTitle()));
    }
}
