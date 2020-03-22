package com.github.yoep.popcorn.view.controllers.desktop.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.media.providers.models.Images;
import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.media.providers.models.Show;
import com.github.yoep.popcorn.messages.MediaMessage;
import com.github.yoep.popcorn.view.controllers.common.components.AbstractCardComponent;
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
    private static final Image POSTER_HOLDER_IMAGE = loadPosterHolderImage();

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
            setPosterHolderImage();

            // try to load the actual image
            Optional.ofNullable(media.getImages())
                    .map(Images::getPoster)
                    .filter(e -> !e.equalsIgnoreCase("n/a"))
                    .ifPresent(this::loadMediaImage);
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

    private void setPosterHolderImage() {
        try {
            // use the post holder as the default image while the media image is being loaded
            setBackgroundImage(POSTER_HOLDER_IMAGE, false);
        } catch (Exception ex) {
            log.error(ex.getMessage(), ex);
        }
    }

    private void loadMediaImage(String mediaImage) {
        try {
            var image = new Image(mediaImage, POSTER_WIDTH, POSTER_HEIGHT, true, true);

            // verify if an error occurred while loading the media image
            // if so, don't replace the poster holder image
            if (!image.isError()) {
                setBackgroundImage(image, true);
            } else {
                var exception = image.getException();
                log.warn("Failed to load media image, " + exception.getMessage(), exception);
            }
        } catch (Exception ex) {
            log.error(ex.getMessage(), ex);
        }
    }

    private static Image loadPosterHolderImage() {
        try {
            var inputStream = getPosterHolderResource().getInputStream();

            return new Image(inputStream, POSTER_WIDTH, POSTER_HEIGHT, true, true);
        } catch (Exception ex) {
            log.error(ex.getMessage(), ex);
        }

        return null;
    }
}
