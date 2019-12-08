package com.github.yoep.popcorn.controllers.sections;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.controls.Stars;
import com.github.yoep.popcorn.messages.DetailsMessage;
import com.github.yoep.popcorn.providers.media.models.Images;
import com.github.yoep.popcorn.providers.media.models.Media;
import com.github.yoep.popcorn.providers.media.models.Movie;
import com.github.yoep.popcorn.providers.media.models.Torrent;
import javafx.application.Application;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Button;
import javafx.scene.control.Label;
import javafx.scene.control.Tooltip;
import javafx.scene.effect.GaussianBlur;
import javafx.scene.image.Image;
import javafx.scene.image.ImageView;
import javafx.scene.input.Clipboard;
import javafx.scene.input.ClipboardContent;
import javafx.scene.input.MouseButton;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.*;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.lang3.StringUtils;
import org.springframework.core.io.ClassPathResource;
import org.springframework.stereotype.Controller;
import org.springframework.util.Assert;

import java.net.URL;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.ResourceBundle;
import java.util.stream.Stream;

@Slf4j
@Controller
@RequiredArgsConstructor
public class DetailsSectionController implements Initializable {
    private final List<DetailsListener> listeners = new ArrayList<>();
    private final LocaleText localeText;
    private final Application application;

    private Media media;

    @FXML
    private BorderPane backgroundImage;
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
    private Button watchTrailerButton;

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializeBackground();
        initializePoster();
        initializeTooltips();
    }

    /**
     * Add a details listener to this controller.
     *
     * @param listener The listener to add.
     */
    public void addListener(DetailsListener listener) {
        Assert.notNull(listener, "listener cannot be null");
        synchronized (listeners) {
            listeners.add(listener);
        }
    }

    public void load(Media media) {
        Assert.notNull(media, "media cannot be null");
        this.media = media;

        loadImages();
        loadText();
        loadStars();
        loadButtons();
    }

    private void initializeBackground() {
        backgroundImage.setEffect(new GaussianBlur(30));
    }

    private void initializePoster() {
        poster.fitHeightProperty().bind(posterHolder.heightProperty());
        poster.fitWidthProperty().bind(posterHolder.widthProperty());
    }

    private void initializeTooltips() {
        Tooltip.install(magnetLink, new Tooltip(localeText.get(DetailsMessage.MAGNET_LINK)));
        Tooltip.install(health, new Tooltip(localeText.get(DetailsMessage.HEALTH_UNKNOWN)));
    }

    private void loadImages() {
        try {
            Image posterImage = Optional.ofNullable(media.getImages())
                    .map(Images::getPoster)
                    .filter(e -> !e.equalsIgnoreCase("n/a"))
                    .map(Image::new)
                    .orElse(new Image(new ClassPathResource("/images/posterholder.png").getInputStream()));

            poster.setImage(posterImage);
        } catch (Exception ex) {
            log.error(ex.getMessage(), ex);
        }

        try {
            Optional.ofNullable(media.getImages())
                    .map(Images::getFanart)
                    .filter(e -> !e.equalsIgnoreCase("n/a"))
                    .map(Image::new)
                    .ifPresent(image -> backgroundImage.setBackground(new Background(
                            new BackgroundImage(image, BackgroundRepeat.NO_REPEAT, BackgroundRepeat.NO_REPEAT, BackgroundPosition.DEFAULT,
                                    new BackgroundSize(BackgroundSize.AUTO, BackgroundSize.AUTO, false, false, true, true))
                    )));
        } catch (Exception ex) {
            log.error(ex.getMessage(), ex);
        }
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
        Tooltip.install(ratingStars, new Tooltip(media.getRating().getPercentage() / 10 + "/10"));
    }

    private void loadButtons() {
        Movie movie = (Movie) media;
        watchTrailerButton.setVisible(StringUtils.isNotEmpty(movie.getTrailer()));
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
    private void close() {
        synchronized (listeners) {
            listeners.forEach(DetailsListener::onClose);
        }

        poster.setImage(null);
    }
}
