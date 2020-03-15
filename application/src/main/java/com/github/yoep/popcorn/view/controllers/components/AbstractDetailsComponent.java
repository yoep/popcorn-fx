package com.github.yoep.popcorn.view.controllers.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.view.controls.Stars;
import com.github.yoep.popcorn.media.providers.models.Images;
import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.media.providers.models.MediaTorrentInfo;
import com.github.yoep.popcorn.messages.DetailsMessage;
import com.github.yoep.popcorn.subtitles.SubtitleService;
import com.github.yoep.popcorn.subtitles.controls.LanguageFlagSelection;
import com.github.yoep.popcorn.subtitles.models.SubtitleInfo;
import com.github.yoep.popcorn.torrent.TorrentService;
import com.github.yoep.popcorn.torrent.models.TorrentHealth;
import javafx.application.Application;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import javafx.scene.control.Tooltip;
import javafx.scene.image.Image;
import javafx.scene.image.ImageView;
import javafx.scene.input.Clipboard;
import javafx.scene.input.ClipboardContent;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.Pane;
import javafx.util.Duration;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.io.ClassPathResource;
import org.springframework.core.task.TaskExecutor;

import java.util.*;
import java.util.stream.Collectors;

/**
 * Abstract definition of a details component for showing {@link Media} information.
 *
 * @param <T> The media type of the details component.
 */
@Slf4j
public abstract class AbstractDetailsComponent<T extends Media> implements Initializable {
    protected static final String LIKED_STYLE_CLASS = "liked";
    protected static final String QUALITY_ACTIVE_CLASS = "active";

    protected final TaskExecutor taskExecutor;
    protected final LocaleText localeText;
    protected final TorrentService torrentService;
    protected final SubtitleService subtitleService;
    protected final Application application;

    protected T media;
    protected SubtitleInfo subtitle;
    protected boolean liked;
    protected String quality;

    @FXML
    protected Pane posterHolder;
    @FXML
    protected ImageView poster;
    @FXML
    protected Stars ratingStars;
    @FXML
    protected Icon favoriteIcon;
    @FXML
    protected Label favoriteText;
    @FXML
    protected Icon magnetLink;
    @FXML
    protected Icon health;
    @FXML
    protected Pane qualitySelectionPane;
    @FXML
    protected LanguageFlagSelection languageSelection;

    //region Constructors

    public AbstractDetailsComponent(TaskExecutor taskExecutor, LocaleText localeText, TorrentService torrentService, SubtitleService subtitleService,
                                    Application application) {
        this.taskExecutor = taskExecutor;
        this.localeText = localeText;
        this.torrentService = torrentService;
        this.subtitleService = subtitleService;
        this.application = application;
    }

    //endregion

    /**
     * Initialize the poster of the details component.
     * This will bind the imageview's "fitTo" properties to the parent size.
     */
    protected void initializePoster() {
        poster.fitHeightProperty().bind(posterHolder.heightProperty());
        poster.fitWidthProperty().bind(posterHolder.widthProperty());
    }

    /**
     * Load the stars component.
     * This will set the rating of the stars that needs to be shown.
     */
    protected void loadStars() {
        ratingStars.setRating(media.getRating());
        Tooltip tooltip = new Tooltip(media.getRating().getPercentage() / 10 + "/10");
        instantTooltip(tooltip);
        Tooltip.install(ratingStars, tooltip);
    }

    protected void loadPosterImage() {
        // load the poster image in the background
        taskExecutor.execute(() -> {
            try {
                final Image posterImage = Optional.ofNullable(media.getImages())
                        .map(Images::getPoster)
                        .filter(e -> !e.equalsIgnoreCase("n/a"))
                        .map(url -> new Image(url, true))
                        .orElse(new Image(new ClassPathResource("/images/posterholder.png").getInputStream()));

                Platform.runLater(() -> poster.setImage(posterImage));
            } catch (Exception ex) {
                log.error(ex.getMessage(), ex);
            }
        });
    }

    protected void loadQualitySelection(Map<String, MediaTorrentInfo> torrents) {
        List<Label> qualities = torrents.keySet().stream()
                .filter(e -> !e.equals("0")) // filter out the 0 quality
                .sorted(Comparator.comparing(o -> Integer.parseInt(o.replaceAll("[a-z]", ""))))
                .map(this::createQualityOption)
                .collect(Collectors.toList());

        // replace the quality selection with the new items
        qualitySelectionPane.getChildren().clear();
        qualitySelectionPane.getChildren().addAll(qualities);

        switchActiveQuality(qualities.get(qualities.size() - 1).getText());
    }

    protected void switchLiked(boolean isLiked) {
        this.liked = isLiked;

        if (isLiked) {
            favoriteIcon.getStyleClass().add(LIKED_STYLE_CLASS);
            favoriteText.setText(localeText.get(DetailsMessage.REMOVE_FROM_BOOKMARKS));
        } else {
            favoriteIcon.getStyleClass().remove(LIKED_STYLE_CLASS);
            favoriteText.setText(localeText.get(DetailsMessage.ADD_TO_BOOKMARKS));
        }
    }

    /**
     * Create a new instant {@link Tooltip} for the given text.
     * This will create a {@link Tooltip} with {@link Tooltip#setShowDelay(Duration)} of {@link Duration#ZERO},
     * {@link Tooltip#setShowDuration(Duration)} of {@link Duration#INDEFINITE},
     * {@link Tooltip#setHideDelay(Duration)} of {@link Duration#ZERO}.
     *
     * @param text The text of the tooltip.
     * @return Returns the instant Tooltip.
     */
    protected Tooltip instantTooltip(String text) {
        return instantTooltip(new Tooltip(text));
    }

    /**
     * Update the given tooltip so it's shown instantly.
     * This will update the {@link Tooltip} with {@link Tooltip#setShowDelay(Duration)} of {@link Duration#ZERO},
     * {@link Tooltip#setShowDuration(Duration)} of {@link Duration#INDEFINITE},
     * {@link Tooltip#setHideDelay(Duration)} of {@link Duration#ZERO}.
     *
     * @param tooltip The tooltip to update.
     * @return Returns same tooltip instance..
     */
    protected Tooltip instantTooltip(Tooltip tooltip) {
        tooltip.setShowDelay(Duration.ZERO);
        tooltip.setShowDuration(Duration.INDEFINITE);
        tooltip.setHideDelay(Duration.ZERO);
        return tooltip;
    }

    protected void switchHealth(MediaTorrentInfo torrentInfo) {
        health.getStyleClass().removeIf(e -> !e.equals("health"));
        TorrentHealth health = torrentService.calculateHealth(torrentInfo.getSeed(), torrentInfo.getPeer());

        this.health.getStyleClass().add(health.getStatus().getStyleClass());
        Tooltip healthTooltip = new Tooltip(getHealthTooltip(torrentInfo, health));
        healthTooltip.setWrapText(true);
        instantTooltip(healthTooltip);
        Tooltip.install(this.health, healthTooltip);
    }

    protected String getHealthTooltip(MediaTorrentInfo torrentInfo, TorrentHealth health) {
        return localeText.get(health.getStatus().getKey()) + " - Ratio: " + String.format("%1$,.2f", health.getRatio()) + "\n" +
                "Seeds: " + torrentInfo.getSeed() + " - Peers: " + torrentInfo.getPeer();
    }

    protected void openMagnetLink(MediaTorrentInfo torrentInfo) {
        try {
            application.getHostServices().showDocument(torrentInfo.getUrl());
        } catch (Exception ex) {
            log.error(ex.getMessage(), ex);
        }
    }

    protected void copyMagnetLink(MediaTorrentInfo torrentInfo) {
        ClipboardContent clipboardContent = new ClipboardContent();
        clipboardContent.putUrl(torrentInfo.getUrl());
        clipboardContent.putString(torrentInfo.getUrl());
        Clipboard.getSystemClipboard().setContent(clipboardContent);
    }

    /**
     * Switch the active quality selection to the given quality value.
     *
     * @param quality The quality value to set as active.
     */
    protected void switchActiveQuality(String quality) {
        this.quality = quality;

        qualitySelectionPane.getChildren().forEach(e -> e.getStyleClass().remove(QUALITY_ACTIVE_CLASS));
        qualitySelectionPane.getChildren().stream()
                .map(e -> (Label) e)
                .filter(e -> e.getText().equalsIgnoreCase(quality))
                .findFirst()
                .ifPresent(e -> e.getStyleClass().add(QUALITY_ACTIVE_CLASS));
    }

    /**
     * Handle the subtitles response from the subtitle service.
     *
     * @param subtitles The subtitles to process.
     * @param throwable the exception error to process.
     */
    protected void handleSubtitlesResponse(List<SubtitleInfo> subtitles, Throwable throwable) {
        if (throwable == null) {
            // filter out all the subtitles that don't have a flag
            final List<SubtitleInfo> filteredSubtitles = subtitles.stream()
                    .filter(e -> e.isNone() || Objects.equals(e.getImdbId(), media.getId()))
                    .sorted()
                    .collect(Collectors.toList());

            Platform.runLater(() -> {
                languageSelection.getItems().clear();
                languageSelection.getItems().addAll(filteredSubtitles);
                languageSelection.select(subtitleService.getDefault(filteredSubtitles));
            });
        } else {
            log.error(throwable.getMessage(), throwable);
        }
    }

    /**
     * Reset the details component information to nothing.
     * This will allow the GC to dispose the items when the media details are no longer needed.
     */
    protected void reset() {
        this.media = null;
        this.subtitle = null;
        this.liked = false;
        this.quality = null;
    }

    /**
     * Reset the language selection to the special type {@link SubtitleInfo#none()}.
     */
    protected void resetLanguageSelection() {
        languageSelection.getItems().clear();
        languageSelection.getItems().add(SubtitleInfo.none());
        languageSelection.select(0);
    }

    //region Functions

    private Label createQualityOption(String quality) {
        Label label = new Label(quality);

        label.getStyleClass().add("quality");
        label.setOnMouseClicked(this::onQualityClicked);

        return label;
    }

    private void onQualityClicked(MouseEvent event) {
        Label label = (Label) event.getSource();

        switchActiveQuality(label.getText());
    }

    //endregion
}
