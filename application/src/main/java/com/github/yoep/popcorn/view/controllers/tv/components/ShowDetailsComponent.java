package com.github.yoep.popcorn.view.controllers.tv.components;

import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.ShowSerieDetailsActivity;
import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.media.providers.models.Show;
import com.github.yoep.popcorn.settings.SettingsService;
import com.github.yoep.popcorn.torrent.TorrentService;
import com.github.yoep.popcorn.view.services.ImageService;
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

    public ShowDetailsComponent(ActivityManager activityManager, TorrentService torrentService, ImageService imageService, SettingsService settingsService) {
        super(imageService, torrentService, settingsService);
        this.activityManager = activityManager;
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    private void init() {
        activityManager.register(ShowSerieDetailsActivity.class, activity ->
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
