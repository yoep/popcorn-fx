package com.github.yoep.popcorn.controllers.components;

import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.CloseDetailsActivity;
import com.github.yoep.popcorn.activities.ShowSerieDetailsActivity;
import com.github.yoep.popcorn.media.providers.models.Show;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.scene.control.Label;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.lang3.StringUtils;
import org.springframework.core.task.TaskExecutor;
import org.springframework.stereotype.Component;

import javax.annotation.PostConstruct;
import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
@Component
public class ShowDetailsComponent extends AbstractDetailsComponent<Show> {
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

    public ShowDetailsComponent(ActivityManager activityManager, TaskExecutor taskExecutor) {
        super(taskExecutor);
        this.activityManager = activityManager;
    }

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializePoster();
    }

    @PostConstruct
    public void init() {
        initializeListeners();
    }

    private void initializeListeners() {
        activityManager.register(ShowSerieDetailsActivity.class, activity -> Platform.runLater(() -> load(activity.getMedia())));
    }

    private void load(Show media) {
        this.media = media;

        loadText();
        loadStars();
        loadPosterImage();
    }

    private void loadText() {
        title.setText(media.getTitle());
        year.setText(media.getYear());
        duration.setText(media.getRuntime() + " min");
        status.setText(media.getStatus());
        genres.setText(String.join(" / ", media.getGenres()));
        overview.setText(media.getSynopsis());
    }

    private void reset() {
        title.setText(StringUtils.EMPTY);
        overview.setText(StringUtils.EMPTY);
        year.setText(StringUtils.EMPTY);
        duration.setText(StringUtils.EMPTY);
        status.setText(StringUtils.EMPTY);
        genres.setText(StringUtils.EMPTY);
        poster.setImage(null);
    }

    @FXML
    private void close() {
        activityManager.register(new CloseDetailsActivity() {
        });
        reset();
    }
}
