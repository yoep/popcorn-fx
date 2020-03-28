package com.github.yoep.popcorn.view.controllers.tv.components;

import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.ShowSerieDetailsActivity;
import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.media.providers.models.Show;
import com.github.yoep.popcorn.torrent.TorrentService;
import com.github.yoep.popcorn.view.services.ImageService;
import javafx.application.Platform;
import javafx.scene.image.Image;
import lombok.extern.slf4j.Slf4j;

import javax.annotation.PostConstruct;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;

@Slf4j
public class ShowDetailsComponent extends AbstractTvDetailsComponent<Show> {
    private static final double POSTER_WIDTH = 198.0;
    private static final double POSTER_HEIGHT = 215.0;

    private final ActivityManager activityManager;

    //region Constructors

    public ShowDetailsComponent(ActivityManager activityManager, TorrentService torrentService, ImageService imageService) {
        super(imageService, torrentService);
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
    }

    @Override
    protected CompletableFuture<Optional<Image>> loadPoster(Media media) {
        return imageService.loadPoster(media, POSTER_WIDTH, POSTER_HEIGHT);
    }

    //endregion
}
