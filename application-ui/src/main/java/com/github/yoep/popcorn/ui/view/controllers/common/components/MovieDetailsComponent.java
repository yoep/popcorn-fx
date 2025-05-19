package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.ShowMovieDetailsEvent;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media;
import com.github.yoep.popcorn.backend.media.MovieDetails;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.subtitles.ISubtitleService;
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

import java.net.URL;
import java.util.Objects;
import java.util.Optional;
import java.util.ResourceBundle;

@Slf4j
public class MovieDetailsComponent extends AbstractDesktopDetailsComponent<MovieDetails> {
    static final String DEFAULT_TORRENT_AUDIO = "en";
    static final String POSTER_COMPONENT_VIEW = "components/poster.component.fxml";
    static final String ACTIONS_COMPONENT_VIEW = "components/movie-actions.component.fxml";

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
                                 ISubtitleService subtitleService,
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
            if (event.getMedia() instanceof MovieDetails movie) {
                Platform.runLater(() -> switchHealth(movie.getTorrents().stream()
                        .filter(e -> Objects.equals(e.getLanguage(), DEFAULT_TORRENT_AUDIO))
                        .map(Media.TorrentLanguage::getTorrents)
                        .map(Media.TorrentQuality::getQualitiesMap)
                        .map(e -> e.get(event.getQuality()))
                        .findFirst()
                        .orElse(null)));
            }
            this.quality = event.getQuality();
            return event;
        });
    }

    private void initializeActions() {
        var pane = viewLoader.load(ACTIONS_COMPONENT_VIEW);
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
        title.setText(null);
        overview.setText(null);
        year.setText(null);
        duration.setText(null);
        genres.setText(null);
    }

    //endregion

    //region Functions

    private void initializePoster() {
        var poster = viewLoader.load(POSTER_COMPONENT_VIEW);
        detailsContent.add(poster, 0, 0);
    }

    private void loadText() {
        title.setText(media.title());
        overview.setText(media.synopsis());
        year.setText(media.year());
        duration.setText(media.runtime() + " min");
        genres.setText(String.join(" / ", media.genres()));
    }

    @FXML
    void onMagnetClicked(MouseEvent event) {
        event.consume();
        media.getTorrents().stream()
                .filter(e -> Objects.equals(e.getLanguage(), DEFAULT_TORRENT_AUDIO))
                .map(Media.TorrentLanguage::getTorrents)
                .map(Media.TorrentQuality::getQualitiesMap)
                .findFirst()
                .flatMap(e -> Optional.ofNullable(e.get(quality)))
                .ifPresent(torrentInfo -> {
                    if (event.getButton() == MouseButton.SECONDARY) {
                        copyMagnetLink(torrentInfo);
                    } else {
                        openMagnetLink(torrentInfo);
                    }
                });
    }

    //endregion
}
