package com.github.yoep.popcorn.controllers.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.CloseDetailsActivity;
import com.github.yoep.popcorn.activities.ShowSerieDetailsActivity;
import com.github.yoep.popcorn.controls.Episodes;
import com.github.yoep.popcorn.media.providers.models.Episode;
import com.github.yoep.popcorn.media.providers.models.Show;
import com.github.yoep.popcorn.messages.DetailsMessage;
import com.github.yoep.popcorn.models.Season;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.scene.control.*;
import javafx.scene.layout.Pane;
import javafx.util.Callback;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.lang3.StringUtils;
import org.springframework.core.task.TaskExecutor;
import org.springframework.stereotype.Component;

import javax.annotation.PostConstruct;
import java.net.URL;
import java.util.Comparator;
import java.util.Objects;
import java.util.ResourceBundle;
import java.util.stream.Collectors;

@Slf4j
@Component
public class ShowDetailsComponent extends AbstractDetailsComponent<Show> {
    private final ActivityManager activityManager;
    private final LocaleText localeText;

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
    @FXML
    private Pane seasonsSection;
    @FXML
    private ListView<Season> seasons;
    @FXML
    private Episodes episodes;

    public ShowDetailsComponent(ActivityManager activityManager, TaskExecutor taskExecutor, LocaleText localeText) {
        super(taskExecutor);
        this.activityManager = activityManager;
        this.localeText = localeText;
    }

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializePoster();
        initializeListViews();
    }

    @PostConstruct
    public void init() {
        initializeListeners();
    }

    private void initializeListeners() {
        activityManager.register(ShowSerieDetailsActivity.class, activity -> Platform.runLater(() -> load(activity.getMedia())));
    }

    private void initializeListViews() {
        seasons.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> switchSeason(newValue));

    }

    private void load(Show media) {
        this.media = media;

        reset();
        loadText();
        loadStars();
        loadSeasons();
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

    private void loadSeasons() {
        seasonsSection.setVisible(media.getNumberOfSeasons() > 1);

        for (int i = 1; i <= media.getNumberOfSeasons(); i++) {
            seasons.getItems().add(new Season(i, localeText.get(DetailsMessage.SEASON, i)));
        }

        seasons.getSelectionModel().select(0);
    }

    private void switchSeason(Season newSeason) {
        episodes.getItems().clear();
        episodes.getItems().addAll(media.getEpisodes().stream()
                .filter(Objects::nonNull)
                .filter(e -> e.getSeason() == newSeason.getSeason())
                .sorted(Comparator.comparing(Episode::getEpisode))
                .collect(Collectors.toList()));
        episodes.getSelectionModel().select(0);
    }

    private void reset() {
        title.setText(StringUtils.EMPTY);
        overview.setText(StringUtils.EMPTY);
        year.setText(StringUtils.EMPTY);
        duration.setText(StringUtils.EMPTY);
        status.setText(StringUtils.EMPTY);
        genres.setText(StringUtils.EMPTY);
        seasons.getItems().clear();
        episodes.getItems().clear();
        poster.setImage(null);
    }

    @FXML
    private void close() {
        activityManager.register(new CloseDetailsActivity() {
        });
        reset();
    }
}
