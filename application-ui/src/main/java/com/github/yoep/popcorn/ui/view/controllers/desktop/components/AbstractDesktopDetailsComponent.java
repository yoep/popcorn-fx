package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.player.adapter.PlayerManagerService;
import com.github.yoep.popcorn.ui.events.OpenMagnetLinkEvent;
import com.github.yoep.popcorn.ui.events.SuccessNotificationEvent;
import com.github.yoep.popcorn.ui.media.favorites.FavoriteService;
import com.github.yoep.popcorn.ui.media.providers.models.Media;
import com.github.yoep.popcorn.ui.media.providers.models.MediaTorrentInfo;
import com.github.yoep.popcorn.ui.media.watched.WatchedService;
import com.github.yoep.popcorn.ui.messages.DetailsMessage;
import com.github.yoep.popcorn.ui.settings.SettingsService;
import com.github.yoep.popcorn.ui.subtitles.SubtitlePickerService;
import com.github.yoep.popcorn.ui.subtitles.SubtitleService;
import com.github.yoep.popcorn.ui.subtitles.controls.LanguageFlagSelection;
import com.github.yoep.popcorn.ui.subtitles.controls.LanguageSelectionListener;
import com.github.yoep.popcorn.ui.subtitles.models.SubtitleInfo;
import com.github.yoep.popcorn.ui.view.controllers.common.components.AbstractDetailsComponent;
import com.github.yoep.popcorn.ui.view.controls.WatchNowButton;
import com.github.yoep.popcorn.ui.view.services.HealthService;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.application.Platform;
import javafx.beans.InvalidationListener;
import javafx.beans.value.ChangeListener;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import javafx.scene.control.Tooltip;
import javafx.scene.input.Clipboard;
import javafx.scene.input.ClipboardContent;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.Pane;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.ApplicationEventPublisher;

import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.stream.Collectors;

/**
 * Abstract definition of a details component for showing {@link Media} information.
 *
 * @param <T> The media type of the details component.
 */
@Slf4j
public abstract class AbstractDesktopDetailsComponent<T extends Media> extends AbstractDetailsComponent<T> implements Initializable {
    protected static final String LIKED_STYLE_CLASS = "liked";
    protected static final String WATCHED_STYLE_CLASS = "seen";
    protected static final String QUALITY_ACTIVE_CLASS = "active";

    protected final ApplicationEventPublisher eventPublisher;
    protected final SubtitleService subtitleService;
    protected final SubtitlePickerService subtitlePickerService;
    protected final FavoriteService favoriteService;
    protected final WatchedService watchedService;
    protected final PlayerManagerService playerService;

    protected SubtitleInfo subtitle;
    protected boolean liked;
    protected String quality;

    private final ChangeListener<Boolean> watchedListener = (observable, oldValue, newValue) -> switchWatched(newValue);
    private final ChangeListener<Boolean> likedListener = (observable, oldValue, newValue) -> switchLiked(newValue);

    @FXML
    protected Icon favoriteIcon;
    @FXML
    protected Icon watchedIcon;
    @FXML
    protected Icon magnetLink;
    @FXML
    protected Pane qualitySelectionPane;
    @FXML
    protected LanguageFlagSelection languageSelection;
    @FXML
    protected WatchNowButton watchNowButton;

    //region Constructors

    protected AbstractDesktopDetailsComponent(ApplicationEventPublisher eventPublisher,
                                              LocaleText localeText,
                                              HealthService healthService,
                                              SubtitleService subtitleService,
                                              SubtitlePickerService subtitlePickerService,
                                              ImageService imageService,
                                              SettingsService settingsService, FavoriteService favoriteService, WatchedService watchedService,
                                              PlayerManagerService playerService) {
        super(localeText, imageService, healthService, settingsService);
        this.eventPublisher = eventPublisher;
        this.subtitleService = subtitleService;
        this.subtitlePickerService = subtitlePickerService;
        this.favoriteService = favoriteService;
        this.watchedService = watchedService;
        this.playerService = playerService;
    }

    //endregion

    //region AbstractDetailsComponent

    @Override
    protected void loadStars() {
        super.loadStars();

        var tooltip = new Tooltip(getRatingText());
        instantTooltip(tooltip);
        Tooltip.install(ratingStars, tooltip);
    }

    //endregion

    //region Functions

    protected void initializeWatchNow() {
        // listen for changes in the players
        playerService.playersProperty().addListener((InvalidationListener) change -> updateExternalPlayers());
        playerService.activePlayerProperty().addListener((observable, oldValue, newValue) -> watchNowButton.select(newValue));

        // create initial list for the current known external players
        updateExternalPlayers();

        // listen on player selection changed
        watchNowButton.selectedItemProperty().addListener((observable, oldValue, newValue) -> {
            // verify if the new value is not null
            // if so, update the active player
            if (newValue != null)
                playerService.setActivePlayer(newValue);
        });
    }

    protected void loadQualitySelection(Map<String, MediaTorrentInfo> torrents) {
        var resolutions = getVideoResolutions(torrents);
        var defaultQuality = getDefaultVideoResolution(resolutions);
        var qualities = resolutions.stream()
                .map(this::createQualityOption)
                .collect(Collectors.toList());

        // replace the quality selection with the new items
        qualitySelectionPane.getChildren().clear();
        qualitySelectionPane.getChildren().addAll(qualities);

        switchActiveQuality(defaultQuality);
    }

    protected void loadFavoriteAndWatched() {
        switchLiked(favoriteService.isLiked(media));
        switchWatched(watchedService.isWatched(media));

        media.watchedProperty().addListener(watchedListener);
        media.likedProperty().addListener(likedListener);
    }

    protected void switchWatched(boolean isWatched) {
        if (isWatched) {
            watchedIcon.setText(Icon.CHECK_UNICODE);
            watchedIcon.getStyleClass().add(WATCHED_STYLE_CLASS);
        } else {
            watchedIcon.setText(Icon.EYE_SLASH_UNICODE);
            watchedIcon.getStyleClass().remove(WATCHED_STYLE_CLASS);
        }
    }

    protected void switchLiked(boolean isLiked) {
        this.liked = isLiked;

        if (isLiked) {
            favoriteIcon.getStyleClass().add(LIKED_STYLE_CLASS);
        } else {
            favoriteIcon.getStyleClass().remove(LIKED_STYLE_CLASS);
        }
    }

    protected void openMagnetLink(MediaTorrentInfo torrentInfo) {
        eventPublisher.publishEvent(new OpenMagnetLinkEvent(this, torrentInfo.getUrl()));
    }

    protected void copyMagnetLink(MediaTorrentInfo torrentInfo) {
        ClipboardContent clipboardContent = new ClipboardContent();
        clipboardContent.putUrl(torrentInfo.getUrl());
        clipboardContent.putString(torrentInfo.getUrl());
        Clipboard.getSystemClipboard().setContent(clipboardContent);

        eventPublisher.publishEvent(new SuccessNotificationEvent(this, localeText.get(DetailsMessage.MAGNET_LINK_COPIED_TO_CLIPBOARD)));
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
        Platform.runLater(() -> languageSelection.setLoading(false));

        if (throwable == null) {
            // filter out all the subtitles that don't have a flag
            final List<SubtitleInfo> filteredSubtitles = subtitles.stream()
                    .filter(e -> e.isNone() || e.isCustom() || Objects.equals(e.getImdbId(), media.getId()))
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

    protected LanguageSelectionListener createLanguageListener() {
        return newValue -> {
            if (newValue.isCustom()) {
                onCustomSubtitleSelected();
            } else {
                this.subtitle = newValue;
            }
        };
    }

    /**
     * Reset the details component information to nothing.
     * This will allow the GC to dispose the items when the media details are no longer needed.
     */
    protected void reset() {
        if (media != null) {
            media.watchedProperty().removeListener(watchedListener);
            media.likedProperty().removeListener(likedListener);
        }

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

    private void updateExternalPlayers() {
        Platform.runLater(() -> {
            watchNowButton.clear();
            watchNowButton.addItems(playerService.getPlayers());
            watchNowButton.select(playerService.getActivePlayer().orElse(null));
        });
    }

    private void onCustomSubtitleSelected() {
        Platform.runLater(() -> {
            var subtitleInfo = subtitlePickerService.pickCustomSubtitle();

            // if a custom subtitle was picked by the user, update the subtitle with the custom subtitle
            // otherwise, the subtitle pick was cancelled and we need to reset the selected language to disabled
            subtitleInfo.ifPresentOrElse(subtitle -> this.subtitle = subtitle, () -> languageSelection.select(SubtitleInfo.none()));
        });
    }

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
