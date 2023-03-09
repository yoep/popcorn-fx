package com.github.yoep.popcorn.ui.view.controllers.components;

import com.github.spring.boot.javafx.stereotype.ViewController;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.ShowMovieDetailsEvent;
import com.github.yoep.popcorn.backend.events.WatchNowEvent;
import com.github.yoep.popcorn.backend.media.providers.models.MovieDetails;
import com.github.yoep.popcorn.backend.messages.SubtitleMessage;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.subtitles.SubtitlePickerService;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import com.github.yoep.popcorn.ui.controls.LanguageFlagCell;
import com.github.yoep.popcorn.ui.events.LoadMediaTorrentEvent;
import com.github.yoep.popcorn.ui.view.services.DetailsComponentService;
import com.github.yoep.popcorn.ui.view.services.HealthService;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.scene.control.Label;
import javafx.scene.control.Tooltip;
import javafx.scene.image.Image;
import javafx.scene.image.ImageView;
import javafx.scene.input.MouseButton;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.GridPane;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.lang3.StringUtils;

import java.io.IOException;
import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
@ViewController
public class MovieDetailsComponent extends AbstractDesktopDetailsComponent<MovieDetails> {
    private static final String DEFAULT_TORRENT_AUDIO = "en";

    private final ViewLoader viewLoader;

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
                                 FxLib fxLib,
                                 ViewLoader viewLoader) {
        super(eventPublisher,
                localeText,
                healthService,
                subtitleService,
                subtitlePickerService,
                imageService,
                settingsService,
                service,
                fxLib);

        this.viewLoader = viewLoader;
    }

    //endregion

    //region Initializable

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        super.initialize(location, resources);
        initializeTooltips();
        initializeLanguageSelection();
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
        eventPublisher.register(WatchNowEvent.class, event -> {
            startMediaPlayback();
            return event;
        });
    }

    private void initializeActions() {
        var pane = viewLoader.load("components/movie-actions.component.fxml");
        GridPane.setColumnIndex(pane, 0);
        GridPane.setRowIndex(pane, 4);
        detailsDescription.getChildren().add(4, pane);
    }

    //endregion

    //region AbstractDetailsComponent

    @Override
    protected void load(MovieDetails media) {
        super.load(media);

        loadText();
        loadSubtitles();
        loadQualitySelection(media.getTorrents().get(DEFAULT_TORRENT_AUDIO));
    }

    @Override
    protected void reset() {
        super.reset();
        resetLanguageSelection();

        title.setText(StringUtils.EMPTY);
        overview.setText(StringUtils.EMPTY);
        year.setText(StringUtils.EMPTY);
        duration.setText(StringUtils.EMPTY);
        genres.setText(StringUtils.EMPTY);
        qualitySelectionPane.getChildren().clear();
    }

    //endregion

    //region Functions

    private void initializeLanguageSelection() {
        languageSelection.setFactory(new LanguageFlagCell() {
            @Override
            public void updateItem(SubtitleInfo item) {
                if (item == null)
                    return;

                setText(null);

                try {
                    var language = item.getLanguage().getNativeName();
                    var image = new ImageView(new Image(item.getFlagResource().getInputStream()));

                    image.setFitHeight(20);
                    image.setPreserveRatio(true);

                    if (item.isNone()) {
                        language = localeText.get(SubtitleMessage.NONE);
                    } else if (item.isCustom()) {
                        language = localeText.get(SubtitleMessage.CUSTOM);
                    }

                    var tooltip = new Tooltip(language);

                    instantTooltip(tooltip);
                    Tooltip.install(image, tooltip);

                    setGraphic(image);
                } catch (IOException ex) {
                    log.error(ex.getMessage(), ex);
                }
            }
        });

        languageSelection.addListener(createLanguageListener());
        resetLanguageSelection();
    }

    private void initializePoster() {
        var poster = viewLoader.load("components/movie-poster.component.fxml");
        detailsContent.add(poster, 0, 0);
    }

    private void loadText() {
        title.setText(media.getTitle());
        overview.setText(media.getSynopsis());
        year.setText(media.getYear());
        duration.setText(media.getRuntime() + " min");
        genres.setText(String.join(" / ", media.getGenres()));
    }

    private void loadSubtitles() {
        resetLanguageSelection();
        languageSelection.setLoading(true);
        subtitleService.retrieveSubtitles(media).whenComplete(this::handleSubtitlesResponse);
    }

    @Override
    protected void switchActiveQuality(String quality) {
        Platform.runLater(() -> {
            super.switchActiveQuality(quality);
            switchHealth(media.getTorrents().get(DEFAULT_TORRENT_AUDIO).get(quality));
        });
    }

    private void startMediaPlayback() {
        var mediaTorrentInfo = media.getTorrents().get(DEFAULT_TORRENT_AUDIO).get(quality);
        eventPublisher.publishEvent(new LoadMediaTorrentEvent(this, mediaTorrentInfo, media, null, quality, subtitle));
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

    @FXML
    void onSubtitleLabelClicked(MouseEvent event) {
        event.consume();
        languageSelection.show();
    }

    //endregion
}
