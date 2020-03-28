package com.github.yoep.popcorn.view.controllers.common.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.media.providers.models.MediaTorrentInfo;
import com.github.yoep.popcorn.torrent.TorrentService;
import com.github.yoep.popcorn.torrent.models.TorrentHealth;
import com.github.yoep.popcorn.view.controls.BackgroundImageCover;
import com.github.yoep.popcorn.view.controls.Stars;
import com.github.yoep.popcorn.view.services.ImageService;
import javafx.fxml.FXML;
import javafx.scene.image.Image;
import javafx.scene.image.ImageView;
import javafx.scene.layout.Pane;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.io.ClassPathResource;
import org.springframework.util.Assert;

import java.util.Optional;
import java.util.concurrent.CompletableFuture;

@Slf4j
public abstract class AbstractDetailsComponent<T extends Media> {
    private static final String POSTER_HOLDER_URI = "/images/posterholder.png";
    private static final Image POSTER_HOLDER = loadPosterHolder();

    protected final ImageService imageService;
    protected final TorrentService torrentService;

    protected T media;

    @FXML
    protected Icon health;
    @FXML
    protected Stars ratingStars;
    @FXML
    protected Pane posterHolder;
    @FXML
    protected ImageView poster;
    @FXML
    protected BackgroundImageCover backgroundImage;

    //region Constructors

    protected AbstractDetailsComponent(ImageService imageService, TorrentService torrentService) {
        Assert.notNull(imageService, "imageService cannot be null");
        Assert.notNull(torrentService, "torrentService cannot be null");
        this.imageService = imageService;
        this.torrentService = torrentService;
    }

    //endregion

    //region Methods

    /**
     * Load the details of the given {@link Media} item.
     *
     * @param media The media item to load the details of.
     */
    protected void load(T media) {
        Assert.notNull(media, "media cannot be null");
        reset();

        this.media = media;

        loadBackgroundImage();
        loadPosterImage();
        loadStars();
    }

    /**
     * Load the media poster for the given media.
     *
     * @param media The media to load the poster of.
     * @return Returns the completable future of the poster load action.
     */
    protected abstract CompletableFuture<Optional<Image>> loadPoster(Media media);

    /**
     * Load the stars component.
     * This will set the rating of the stars that needs to be shown.
     */
    protected void loadStars() {
        ratingStars.setRating(media.getRating());
    }

    /**
     * Switch the health icon to the current media torrent info.
     *
     * @param torrentInfo The media torrent info to display the health status of.
     * @return Returns the health status.
     */
    protected TorrentHealth switchHealth(MediaTorrentInfo torrentInfo) {
        health.getStyleClass().removeIf(e -> !e.equals("health"));
        var health = torrentService.calculateHealth(torrentInfo.getSeed(), torrentInfo.getPeer());

        this.health.getStyleClass().add(health.getStatus().getStyleClass());

        return health;
    }

    /**
     * Reset the details component back to it's idle state.
     */
    protected void reset() {
        this.media = null;
    }

    //endregion

    //region Functions

    private void loadPosterImage() {
        // set the poster holder as the default image
        poster.setImage(POSTER_HOLDER);

        loadPoster(media).whenComplete((image, throwable) -> {
            if (throwable == null) {
                // replace the poster holder with the actual image if present
                image.ifPresent(e -> poster.setImage(e));
            } else {
                log.error(throwable.getMessage(), throwable);
            }
        });
    }

    private void loadBackgroundImage() {
        backgroundImage.reset();
        imageService.loadFanart(media).whenComplete((bytes, throwable) -> {
            if (throwable == null) {
                bytes.ifPresent(e -> backgroundImage.setBackgroundImage(e));
            } else {
                log.error(throwable.getMessage(), throwable);
            }
        });
    }

    private static Image loadPosterHolder() {
        try {
            var resource = new ClassPathResource(POSTER_HOLDER_URI);

            if (resource.exists()) {
                return new Image(resource.getInputStream());
            } else {
                log.warn("Poster holder url \"{}\" does not exist", POSTER_HOLDER_URI);
            }
        } catch (Exception ex) {
            log.error(ex.getMessage(), ex);
        }

        return null;
    }

    //endregion
}
