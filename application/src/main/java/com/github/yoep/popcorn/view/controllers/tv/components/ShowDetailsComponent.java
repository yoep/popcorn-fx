package com.github.yoep.popcorn.view.controllers.tv.components;

import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.ShowSerieDetailsActivity;
import com.github.yoep.popcorn.media.providers.models.Show;
import com.github.yoep.popcorn.view.services.ImageService;
import javafx.application.Platform;
import lombok.extern.slf4j.Slf4j;
import org.springframework.util.Assert;

import javax.annotation.PostConstruct;

@Slf4j
public class ShowDetailsComponent extends AbstractDetailsComponent<Show> {
    private final ActivityManager activityManager;


    public ShowDetailsComponent(ActivityManager activityManager, ImageService imageService) {
        super(imageService);
        this.activityManager = activityManager;
    }

    //region PostConstruct

    @PostConstruct
    private void init() {
        activityManager.register(ShowSerieDetailsActivity.class, activity ->
                Platform.runLater(() -> load(activity.getMedia())));
    }

    //endregion

    private void load(Show media) {
        Assert.notNull(media, "media cannot be null");
        this.media = media;

        loadBackgroundImage();
    }
}
