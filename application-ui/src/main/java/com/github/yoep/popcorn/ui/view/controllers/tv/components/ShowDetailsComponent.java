package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.ui.events.ActivityManager;
import com.github.yoep.popcorn.ui.events.ShowSerieDetailsEvent;
import com.github.yoep.popcorn.ui.media.providers.models.Media;
import com.github.yoep.popcorn.ui.media.providers.models.Show;
import com.github.yoep.popcorn.ui.settings.SettingsService;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import com.github.yoep.torrent.adapter.TorrentService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.scene.control.Label;
import javafx.scene.image.Image;
import lombok.extern.slf4j.Slf4j;

import javax.annotation.PostConstruct;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;

@Slf4j
public class ShowDetailsComponent extends AbstractTvDetailsComponent<Show> {
    private static final double POSTER_WIDTH = 298.0;
    private static final double POSTER_HEIGHT = 315.0;

    private final ActivityManager activityManager;

    @FXML
    private Label title;
    @FXML
    private Label year;
    @FXML
    private Label duration;
    @FXML
    private Label status;
    @FXML
    private Label genres;
    @FXML
    private Label overview;

    //region Constructors

    public ShowDetailsComponent(LocaleText localeText, ActivityManager activityManager, TorrentService torrentService, ImageService imageService, SettingsService settingsService) {
        super(localeText, imageService, torrentService, settingsService);
        this.activityManager = activityManager;
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    private void init() {
        activityManager.register(ShowSerieDetailsEvent.class, activity ->
                Platform.runLater(() -> load(activity.getMedia())));
    }

    //endregion

    //region AbstractTvDetailsComponent

    @Override
    protected void load(Show media) {
        super.load(media);

        loadText();
    }

    @Override
    protected CompletableFuture<Optional<Image>> loadPoster(Media media) {
        return imageService.loadPoster(media, POSTER_WIDTH, POSTER_HEIGHT);
    }

    //endregion

    //region Functions

    private void loadText() {
        title.setText(media.getTitle());
        year.setText(media.getYear());
        duration.setText(media.getRuntime() + " min");
        status.setText(media.getStatus());
        genres.setText(String.join(" / ", media.getGenres()));
        overview.setText(media.getSynopsis());
    }

    //endregion
}
