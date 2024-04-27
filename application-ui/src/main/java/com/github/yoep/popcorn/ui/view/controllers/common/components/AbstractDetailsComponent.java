package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentHealth;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.providers.models.MediaTorrentInfo;
import com.github.yoep.popcorn.backend.media.providers.models.Rating;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.events.CloseDetailsEvent;
import com.github.yoep.popcorn.ui.view.ViewHelper;
import com.github.yoep.popcorn.ui.view.controls.BackgroundImageCover;
import com.github.yoep.popcorn.ui.view.controls.HealthIcon;
import com.github.yoep.popcorn.ui.view.controls.Stars;
import com.github.yoep.popcorn.ui.view.services.HealthService;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Tooltip;
import lombok.AccessLevel;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.Objects;
import java.util.ResourceBundle;
import java.util.concurrent.CancellationException;

@Slf4j
@RequiredArgsConstructor(access = AccessLevel.PROTECTED)
public abstract class AbstractDetailsComponent<T extends Media> implements Initializable {
    protected final LocaleText localeText;
    protected final ImageService imageService;
    protected final HealthService healthService;
    protected final ApplicationConfig settingsService;
    protected final EventPublisher eventPublisher;

    protected T media;

    @FXML
    HealthIcon health;
    @FXML
    Stars ratingStars;
    @FXML
    BackgroundImageCover backgroundImage;

    //region Methods

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        eventPublisher.register(CloseDetailsEvent.class, event -> {
            Platform.runLater(this::reset);
            return event;
        });
    }

    /**
     * Load the details of the given {@link Media} item.
     *
     * @param media The media item to load the details of.
     */
    protected void load(T media) {
        // always reset the details when a new show is being loaded
        // as the onCloseDetails might have been bypassed through another event
        reset();

        Objects.requireNonNull(media, "media cannot be null");
        this.media = media;

        loadBackgroundImage();
        loadStars();
    }

    /**
     * Load the stars component.
     * This will set the rating of the stars that needs to be shown.
     */
    protected void loadStars() {
        media.getRating().ifPresent(ratingStars::setRating);
    }

    /**
     * Switch the health icon to the current media torrent info.
     *
     * @param torrentInfo The media torrent info to display the health status of.
     */
    protected void switchHealth(MediaTorrentInfo torrentInfo) {
        if (health == null)
            return;

        this.health.setUpdating(true);

        // set the health based on the API information
        var health = healthService.calculateHealth(torrentInfo.getSeed(), torrentInfo.getPeer());
        updateHealthIcon(health);

        // request the real-time health
        healthService.getTorrentHealth(torrentInfo.getUrl()).whenComplete((torrentHealth, throwable) -> {
            this.health.setUpdating(false);

            if (throwable == null) {
                updateHealthIcon(torrentHealth);
            } else if (!(throwable instanceof CancellationException)) {
                // do not remove the health state, keep the original info from the API
                log.error("Failed to retrieve health info, " + throwable.getMessage(), throwable);
            }
        });
    }

    /**
     * Get the rating text to display.
     *
     * @return Returns the rating display text.
     */
    protected String getRatingText() {
        return media.getRating()
                .map(Rating::getPercentage)
                .map(percentage -> percentage / 10)
                .map(e -> e + "/10")
                .orElse(null);
    }

    /**
     * Reset the details component back to it's idle state.
     */
    protected void reset() {
        this.media = null;
    }

    //endregion

    //region Functions

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

    private void updateHealthIcon(TorrentHealth health) {
        var healthTooltip = new Tooltip(getHealthTooltip(health));

        healthTooltip.setWrapText(true);
        ViewHelper.instantTooltip(healthTooltip);
        Tooltip.install(this.health, healthTooltip);

        if (removeHealthState()) {
            log.trace("Health states have been removed from the health icon");
        }

        this.health.getStyleClass().add(health.getState().getStyleClass());
    }

    private boolean removeHealthState() {
        return this.health.getStyleClass().removeIf(e -> !e.equals("health"));
    }

    private String getHealthTooltip(TorrentHealth health) {
        return localeText.get(health.getState().getKey()) + " - Ratio: " + String.format("%1$,.2f", health.getRatio()) + "\n" +
                "Seeds: " + health.getSeeds() + " - Peers: " + health.getPeers();
    }

    //endregion
}
