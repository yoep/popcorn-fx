package com.github.yoep.popcorn.view.controllers.tv.components;

import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.ShowMovieDetailsActivity;
import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.media.providers.models.Movie;
import com.github.yoep.popcorn.view.services.ImageService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import javafx.scene.image.Image;
import lombok.extern.slf4j.Slf4j;

import javax.annotation.PostConstruct;
import java.net.URL;
import java.util.Optional;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;

@Slf4j
public class MovieDetailsComponent extends AbstractTvDetailsComponent<Movie> implements Initializable {
    private final ActivityManager activityManager;

    @FXML
    private Label title;
    @FXML
    private Label overview;
    @FXML
    private Label year;
    @FXML
    private Label duration;
    @FXML
    private Label genres;

    //region Constructors

    public MovieDetailsComponent(ActivityManager activityManager, ImageService imageService) {
        super(imageService);
        this.activityManager = activityManager;
    }

    //endregion

    //region Initializable

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializePoster();
    }

    private void initializePoster() {
        poster.fitHeightProperty().bind(posterHolder.heightProperty());
        poster.fitWidthProperty().bind(posterHolder.widthProperty());
    }

    //endregion

    //region AbstractTvDetailsComponent

    @Override
    protected void load(Movie media) {
        super.load(media);

        loadText();
    }

    @Override
    protected CompletableFuture<Optional<Image>> loadPoster(Media media) {
        return imageService.loadPoster(media);
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    private void init() {
        activityManager.register(ShowMovieDetailsActivity.class, activity ->
                Platform.runLater(() -> load(activity.getMedia())));
    }

    //endregion

    //region Functions

    private void loadText() {
        title.setText(media.getTitle());
        overview.setText(media.getSynopsis());
        year.setText(media.getYear());
        duration.setText(media.getRuntime() + " min");
        genres.setText(String.join(" / ", media.getGenres()));
    }

    //endregion
}
