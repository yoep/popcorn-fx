package com.github.yoep.popcorn.controllers.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.activities.*;
import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.media.providers.models.Movie;
import com.github.yoep.popcorn.media.providers.models.Torrent;
import com.github.yoep.popcorn.messages.DetailsMessage;
import com.github.yoep.popcorn.models.TorrentHealth;
import com.github.yoep.popcorn.services.TorrentService;
import javafx.application.Application;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.scene.control.Button;
import javafx.scene.control.Label;
import javafx.scene.control.SplitMenuButton;
import javafx.scene.control.Tooltip;
import javafx.scene.input.Clipboard;
import javafx.scene.input.ClipboardContent;
import javafx.scene.input.MouseButton;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.Pane;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.lang3.StringUtils;
import org.springframework.core.task.TaskExecutor;
import org.springframework.stereotype.Component;
import org.springframework.util.Assert;

import javax.annotation.PostConstruct;
import java.net.URL;
import java.util.Comparator;
import java.util.List;
import java.util.Optional;
import java.util.ResourceBundle;
import java.util.stream.Collectors;
import java.util.stream.Stream;

@Slf4j
@Component
public class MovieDetailsComponent extends AbstractDetailsComponent<Movie> {
    private static final String DEFAULT_TORRENT_AUDIO = "en";

    private final ActivityManager activityManager;
    private final LocaleText localeText;
    private final Application application;
    private final TorrentService torrentService;

    private Tooltip healthTooltip;
    private String quality;

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
    @FXML
    private Icon magnetLink;
    @FXML
    private Icon health;
    @FXML
    private SplitMenuButton watchNowButton;
    @FXML
    private Button watchTrailerButton;
    @FXML
    private Pane qualitySelectionPane;

    public MovieDetailsComponent(ActivityManager activityManager, LocaleText localeText, Application application, TaskExecutor taskExecutor, TorrentService torrentService) {
        super(taskExecutor);
        this.activityManager = activityManager;
        this.localeText = localeText;
        this.application = application;
        this.torrentService = torrentService;
    }

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializePoster();
        initializeTooltips();
    }

    @PostConstruct
    private void init() {
        initializeListeners();
    }

    private void initializeTooltips() {
        Tooltip tooltip = new Tooltip(localeText.get(DetailsMessage.MAGNET_LINK));
        setInstantTooltip(tooltip);
        Tooltip.install(magnetLink, tooltip);
    }

    private void initializeListeners() {
        activityManager.register(ShowMovieDetailsActivity.class, activity ->
                Platform.runLater(() -> load(activity.getMedia())));
    }

    private void reset() {
        title.setText(StringUtils.EMPTY);
        overview.setText(StringUtils.EMPTY);
        year.setText(StringUtils.EMPTY);
        duration.setText(StringUtils.EMPTY);
        genres.setText(StringUtils.EMPTY);
        qualitySelectionPane.getChildren().clear();
        poster.setImage(null);
    }

    private void load(Movie media) {
        Assert.notNull(media, "media cannot be null");
        this.media = media;

        loadText();
        loadStars();
        loadButtons();
        loadQualitySelection();
        loadPosterImage();
    }

    private void loadText() {
        title.setText(media.getTitle());
        overview.setText(media.getSynopsis());
        year.setText(media.getYear());
        duration.setText(media.getRuntime() + " min");
        genres.setText(String.join(" / ", media.getGenres()));
    }

    private void loadButtons() {
        watchTrailerButton.setVisible(StringUtils.isNotEmpty(media.getTrailer()));
    }

    private void loadQualitySelection() {
        List<Label> qualities = media.getTorrents().get(DEFAULT_TORRENT_AUDIO).keySet().stream()
                .sorted(Comparator.comparing(o -> Integer.parseInt(o.replaceAll("[a-z]", ""))))
                .map(this::createQualityOption)
                .collect(Collectors.toList());

        qualitySelectionPane.getChildren().addAll(qualities);
        switchActiveQuality(qualities.get(qualities.size() - 1).getText());
    }

    private String getHealthTooltip(Torrent torrent, TorrentHealth health) {
        return localeText.get(health.getStatus().getKey()) + " - Ratio: " + String.format("%1$,.2f", health.getRatio()) + "\n" +
                "Seeds: " + torrent.getSeed() + " - Peers: " + torrent.getPeer();
    }

    private Label createQualityOption(String quality) {
        Label label = new Label(quality);

        label.getStyleClass().add("quality");
        label.setOnMouseClicked(this::onQualityClicked);

        return label;
    }

    private void openMagnetLink(Torrent torrent) {
        try {
            application.getHostServices().showDocument(torrent.getUrl());
        } catch (Exception ex) {
            log.error(ex.getMessage(), ex);
        }
    }

    private void copyMagnetLink(Torrent torrent) {
        ClipboardContent clipboardContent = new ClipboardContent();
        clipboardContent.putUrl(torrent.getUrl());
        clipboardContent.putString(torrent.getUrl());
        Clipboard.getSystemClipboard().setContent(clipboardContent);
    }

    private void switchHealth() {
        health.getStyleClass().removeIf(e -> !e.equals("health"));

        Torrent torrent = media.getTorrents().get(DEFAULT_TORRENT_AUDIO).get(quality);

        TorrentHealth health = torrentService.calculateHealth(torrent.getSeed(), torrent.getPeer());

        this.health.getStyleClass().add(health.getStatus().getStyleClass());
        this.healthTooltip = new Tooltip(getHealthTooltip(torrent, health));
        this.healthTooltip.setWrapText(true);
        setInstantTooltip(this.healthTooltip);
        Tooltip.install(this.health, this.healthTooltip);
    }

    private void switchActiveQuality(String quality) {
        String activeStyle = "active";

        this.quality = quality;

        qualitySelectionPane.getChildren().forEach(e -> e.getStyleClass().remove(activeStyle));
        qualitySelectionPane.getChildren().stream()
                .map(e -> (Label) e)
                .filter(e -> e.getText().equalsIgnoreCase(quality))
                .findFirst()
                .ifPresent(e -> e.getStyleClass().add(activeStyle));

        switchHealth();
    }

    private void onQualityClicked(MouseEvent event) {
        Label label = (Label) event.getSource();

        switchActiveQuality(label.getText());
    }

    @FXML
    private void onMagnetClicked(MouseEvent event) {
        Optional<Torrent> torrent = media.getTorrents().values().stream()
                .findFirst()
                .map(e -> e.values().stream())
                .flatMap(Stream::findFirst);

        if (event.getButton() == MouseButton.SECONDARY) {
            torrent.ifPresent(this::copyMagnetLink);
        } else {
            torrent.ifPresent(this::openMagnetLink);
        }
    }

    @FXML
    private void onWatchNowClicked() {
        activityManager.register(new LoadMovieActivity() {
            @Override
            public String getQuality() {
                return quality;
            }

            @Override
            public Media getMedia() {
                return media;
            }
        });
    }

    @FXML
    private void onTrailerClicked() {
        activityManager.register(new PlayVideoActivity() {
            @Override
            public String getUrl() {
                return media.getTrailer();
            }

            @Override
            public Optional<String> getQuality() {
                return Optional.empty();
            }

            @Override
            public Media getMedia() {
                return media;
            }
        });
    }

    @FXML
    private void close() {
        reset();
        activityManager.register(new CloseDetailsActivity() {
        });
    }
}
