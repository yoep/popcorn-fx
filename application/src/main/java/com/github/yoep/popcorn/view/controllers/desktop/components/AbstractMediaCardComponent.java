package com.github.yoep.popcorn.view.controllers.desktop.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.media.providers.models.Show;
import com.github.yoep.popcorn.messages.MediaMessage;
import com.github.yoep.popcorn.view.controllers.common.components.AbstractCardComponent;
import com.github.yoep.popcorn.view.services.ImageService;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import javafx.scene.control.Tooltip;
import javafx.scene.image.Image;
import lombok.extern.slf4j.Slf4j;
import org.springframework.util.Assert;

import java.net.URL;
import java.text.MessageFormat;
import java.util.ResourceBundle;

@Slf4j
public abstract class AbstractMediaCardComponent extends AbstractCardComponent implements Initializable {
    private static final Image POSTER_HOLDER_IMAGE = loadPosterHolderImage();

    protected final Media media;
    protected final LocaleText localeText;
    protected final ImageService imageService;

    @FXML
    protected Label title;
    @FXML
    protected Label year;
    @FXML
    protected Label seasons;

    //region Constructors

    protected AbstractMediaCardComponent(Media media, LocaleText localeText, ImageService imageService) {
        Assert.notNull(media, "media cannot be null");
        Assert.notNull(localeText, "localeText cannot be null");
        Assert.notNull(imageService, "imageService cannot be null");
        this.media = media;
        this.localeText = localeText;
        this.imageService = imageService;
    }

    //endregion

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeImage();
        initializeText();
    }

    protected void initializeImage() {
        setPosterHolderImage();
        imageService.loadPoster(media, POSTER_WIDTH, POSTER_HEIGHT).whenComplete((image, throwable) -> {
            if (throwable == null) {
                image.ifPresent(e -> setBackgroundImage(e, true));
            } else {
                log.error(throwable.getMessage(), throwable);
            }
        });
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

    private static Image loadPosterHolderImage() {
        try {
            var inputStream = getPosterHolderResource().getInputStream();
            var image = new Image(inputStream, POSTER_WIDTH, POSTER_HEIGHT, true, true);

            if (!image.isError()) {
                return image;
            } else {
                handleImageError(image);
            }
        } catch (Exception ex) {
            log.error(ex.getMessage(), ex);
        }

        return null;
    }

    private static void handleImageError(Image image) {
        var exception = image.getException();
        var message = MessageFormat.format("Failed to load image card poster url \"{0}\", {1}", image.getUrl(), exception.getMessage());

        log.warn(message, exception);
    }
}
