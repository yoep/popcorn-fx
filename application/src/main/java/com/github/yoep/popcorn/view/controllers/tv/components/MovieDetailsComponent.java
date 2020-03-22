package com.github.yoep.popcorn.view.controllers.tv.components;

import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.ShowMovieDetailsActivity;
import com.github.yoep.popcorn.media.providers.models.Movie;
import com.github.yoep.popcorn.view.services.ImageService;
import javafx.application.Platform;
import lombok.extern.slf4j.Slf4j;
import org.springframework.util.Assert;

import javax.annotation.PostConstruct;

@Slf4j
public class MovieDetailsComponent extends AbstractDetailsComponent<Movie> {
    private final ActivityManager activityManager;

    public MovieDetailsComponent(ActivityManager activityManager, ImageService imageService) {
        super(imageService);
        this.activityManager = activityManager;
    }

    //region PostConstruct

    @PostConstruct
    private void init() {
        activityManager.register(ShowMovieDetailsActivity.class, activity ->
                Platform.runLater(() -> load(activity.getMedia())));
    }

    //endregion

    private void load(Movie media) {
        Assert.notNull(media, "media cannot be null");
        this.media = media;

        loadBackgroundImage();
    }
}
