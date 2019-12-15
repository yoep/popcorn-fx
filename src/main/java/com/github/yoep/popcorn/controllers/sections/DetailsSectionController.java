package com.github.yoep.popcorn.controllers.sections;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.activities.*;
import com.github.yoep.popcorn.controls.Stars;
import com.github.yoep.popcorn.media.providers.MediaException;
import com.github.yoep.popcorn.media.providers.models.Images;
import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.media.providers.models.Movie;
import com.github.yoep.popcorn.media.providers.models.Torrent;
import com.github.yoep.popcorn.messages.DetailsMessage;
import com.github.yoep.popcorn.models.TorrentHealth;
import com.github.yoep.popcorn.services.TorrentService;
import javafx.application.Application;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Button;
import javafx.scene.control.Label;
import javafx.scene.control.SplitMenuButton;
import javafx.scene.control.Tooltip;
import javafx.scene.image.Image;
import javafx.scene.image.ImageView;
import javafx.scene.input.Clipboard;
import javafx.scene.input.ClipboardContent;
import javafx.scene.input.MouseButton;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.BorderPane;
import javafx.util.Duration;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.lang3.StringUtils;
import org.springframework.core.io.ClassPathResource;
import org.springframework.core.task.TaskExecutor;
import org.springframework.stereotype.Controller;
import org.springframework.util.Assert;

import javax.annotation.PostConstruct;
import java.net.URL;
import java.util.Map;
import java.util.Optional;
import java.util.ResourceBundle;
import java.util.stream.Stream;

@Slf4j
@Controller
@RequiredArgsConstructor
public class DetailsSectionController implements Initializable {
    private final ActivityManager activityManager;
    private final LocaleText localeText;
    private final Application application;
    private final TaskExecutor taskExecutor;
    private final TorrentService torrentService;

    private Media media;
    private Tooltip healthTooltip;

    @FXML
    private BorderPane posterHolder;
    @FXML
    private ImageView poster;
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
    private Stars ratingStars;
    @FXML
    private Icon magnetLink;
    @FXML
    private Icon health;
    @FXML
    private SplitMenuButton watchNowButton;
    @FXML
    private Button watchTrailerButton;

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializePoster();
        initializeTooltips();
    }

    @PostConstruct
    private void init() {
        initializeListeners();
    }

    private void initializePoster() {
        poster.fitHeightProperty().bind(posterHolder.heightProperty());
        poster.fitWidthProperty().bind(posterHolder.widthProperty());
    }

    private void initializeTooltips() {
        Tooltip tooltip = new Tooltip(localeText.get(DetailsMessage.MAGNET_LINK));
        setInstantTooltip(tooltip);
        Tooltip.install(magnetLink, tooltip);
    }

    private void initializeListeners() {
        activityManager.register(DetailsShowActivity.class, activity ->
                Platform.runLater(() -> load(activity.getMedia())));
        activityManager.register(DetailsCloseActivity.class, activity -> reset());
    }

    private void reset() {
        Platform.runLater(() -> {
            title.setText(StringUtils.EMPTY);
            overview.setText(StringUtils.EMPTY);
            year.setText(StringUtils.EMPTY);
            duration.setText(StringUtils.EMPTY);
            genres.setText(StringUtils.EMPTY);
            poster.setImage(null);
        });
    }

    private void load(Media media) {
        Assert.notNull(media, "media cannot be null");
        this.media = media;

        loadText();
        loadStars();
        loadButtons();
        loadHealth();
        loadPosterImage();
    }

    private void loadPosterImage() {
        // load the poster image in the background
        taskExecutor.execute(() -> {
            try {
                final Image posterImage = Optional.ofNullable(media.getImages())
                        .map(Images::getPoster)
                        .filter(e -> !e.equalsIgnoreCase("n/a"))
                        .map(Image::new)
                        .orElse(new Image(new ClassPathResource("/images/posterholder.png").getInputStream()));

                Platform.runLater(() -> poster.setImage(posterImage));
            } catch (Exception ex) {
                log.error(ex.getMessage(), ex);
            }
        });
    }

    private void loadText() {
        title.setText(media.getTitle());
        overview.setText(media.getSynopsis());
        year.setText(media.getYear());
        duration.setText(((Movie) media).getRuntime() + " min");
        genres.setText(String.join(" / ", media.getGenres()));
    }

    private void loadStars() {
        ratingStars.setRating(media.getRating());
        Tooltip tooltip = new Tooltip(media.getRating().getPercentage() / 10 + "/10");
        setInstantTooltip(tooltip);
        Tooltip.install(ratingStars, tooltip);
    }

    private void loadButtons() {
        if (watchTrailerButton == null)
            return;

        Movie movie = (Movie) media;
        watchTrailerButton.setVisible(StringUtils.isNotEmpty(movie.getTrailer()));
    }

    private void loadHealth() {
        health.getStyleClass().removeIf(e -> !e.equals("health"));

        media.getTorrents().get("en").entrySet().stream()
                .findFirst()
                .map(Map.Entry::getValue)
                .ifPresent(torrent -> {
                    TorrentHealth health = torrentService.calculateHealth(torrent.getSeed(), torrent.getPeer());

                    this.health.getStyleClass().add(health.getStatus().getStyleClass());
                    this.healthTooltip = new Tooltip(getHealthTooltip(torrent, health));
                    this.healthTooltip.setWrapText(true);
                    setInstantTooltip(this.healthTooltip);
                    Tooltip.install(this.health, this.healthTooltip);
                });

    }

    private String getHealthTooltip(Torrent torrent, TorrentHealth health) {
        return localeText.get(health.getStatus().getKey()) + " - Ratio: " + String.format("%1$,.2f", health.getRatio()) + "\n" +
                "Seeds: " + torrent.getSeed() + " - Peers: " + torrent.getPeer();
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

    private void setInstantTooltip(Tooltip tooltip) {
        tooltip.setShowDelay(Duration.ZERO);
        tooltip.setShowDuration(Duration.INDEFINITE);
        tooltip.setHideDelay(Duration.ZERO);
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
                return media.getTorrents().get("en").keySet().stream()
                        .findFirst()
                        .orElseThrow(() -> new MediaException(media, "No torrents available for the media"));
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
                Movie movie = (Movie) media;
                return movie.getTrailer();
            }

            @Override
            public Media getMedia() {
                return media;
            }
        });
    }

    @FXML
    private void close() {
        activityManager.register(new DetailsCloseActivity() {
        });
        poster.setImage(null);
    }
}
