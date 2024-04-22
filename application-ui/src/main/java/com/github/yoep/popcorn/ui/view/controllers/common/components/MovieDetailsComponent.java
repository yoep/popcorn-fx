package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.ShowMovieDetailsEvent;
import com.github.yoep.popcorn.backend.media.providers.models.MovieDetails;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.events.MediaQualityChangedEvent;
import com.github.yoep.popcorn.ui.view.ViewLoader;
import com.github.yoep.popcorn.ui.view.services.DetailsComponentService;
import com.github.yoep.popcorn.ui.view.services.HealthService;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import com.github.yoep.popcorn.ui.view.services.SubtitlePickerService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.scene.control.Label;
import javafx.scene.input.MouseButton;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.GridPane;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.lang3.StringUtils;

import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
public class MovieDetailsComponent extends AbstractDesktopDetailsComponent<MovieDetails> {
    private static final String DEFAULT_TORRENT_AUDIO = "en";

    private final ViewLoader viewLoader;

    String quality;

    @FXML
    GridPane detailsContent;
    @FXML
    GridPane detailsDescription;
    @FXML
    Label title;
    @FXML
    Label overview;
    @FXML
    Label year;
    @FXML
    Label duration;
    @FXML
    Label genres;

    //region Constructors

    public MovieDetailsComponent(EventPublisher eventPublisher,
                                 LocaleText localeText,
                                 HealthService healthService,
                                 SubtitleService subtitleService,
                                 SubtitlePickerService subtitlePickerService,
                                 ImageService imageService,
                                 ApplicationConfig settingsService,
                                 DetailsComponentService service,
                                 ViewLoader viewLoader) {
        super(eventPublisher,
                localeText,
                healthService,
                subtitleService,
                subtitlePickerService,
                imageService,
                settingsService,
                service);
        this.viewLoader = viewLoader;
    }

    //endregion

    //region Initializable

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        super.initialize(location, resources);
        initializeTooltips();
        initializePoster();
        initializeActions();
        initializeListeners();

        AnchorPane.setLeftAnchor(detailsContent, service.isTvMode() ? 150.0 : 75.0);
    }

    private void initializeListeners() {
        eventPublisher.register(ShowMovieDetailsEvent.class, event -> {
            Platform.runLater(() -> load(event.getMedia()));
            return event;
        });
        eventPublisher.register(MediaQualityChangedEvent.class, event -> {
            Platform.runLater(() -> {
                if (event.getMedia() instanceof MovieDetails movie) {
                    switchHealth(movie.getTorrents().get(DEFAULT_TORRENT_AUDIO).get(event.getQuality()));
                }
            });
            this.quality = event.getQuality();
            return event;
        });
    }

    private void initializeActions() {
        var pane = viewLoader.load("components/movie-actions.component.fxml");
        GridPane.setColumnIndex(pane, 0);
        GridPane.setColumnSpan(pane, 3);
        GridPane.setRowIndex(pane, 4);
        detailsDescription.getChildren().add(4, pane);
    }

    //endregion

    //region AbstractDetailsComponent

    @Override
    protected void load(MovieDetails media) {
        super.load(media);
        loadText();
    }

    @Override
    protected void reset() {
        super.reset();
        title.setText(StringUtils.EMPTY);
        overview.setText(StringUtils.EMPTY);
        year.setText(StringUtils.EMPTY);
        duration.setText(StringUtils.EMPTY);
        genres.setText(StringUtils.EMPTY);
    }

    //endregion

    //region Functions

    private void initializePoster() {
        var poster = viewLoader.load("components/poster.component.fxml");
        detailsContent.add(poster, 0, 0);
    }

    private void loadText() {
        title.setText(media.getTitle());
        overview.setText(media.getSynopsis());
        year.setText(media.getYear());
        duration.setText(media.getRuntime() + " min");
        genres.setText(String.join(" / ", media.getGenres()));
    }

    @FXML
    void onMagnetClicked(MouseEvent event) {
        event.consume();
        var torrentInfo = media.getTorrents().get(DEFAULT_TORRENT_AUDIO).get(quality);

        if (event.getButton() == MouseButton.SECONDARY) {
            copyMagnetLink(torrentInfo);
        } else {
            openMagnetLink(torrentInfo);
        }
    }

    //endregion
}
