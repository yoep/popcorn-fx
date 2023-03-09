package com.github.yoep.popcorn.ui.view.controllers.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.providers.models.MediaTorrentInfo;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.subtitles.SubtitlePickerService;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.backend.subtitles.listeners.LanguageSelectionListener;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import com.github.yoep.popcorn.ui.controls.LanguageFlagSelection;
import com.github.yoep.popcorn.ui.events.OpenMagnetLinkEvent;
import com.github.yoep.popcorn.ui.events.SuccessNotificationEvent;
import com.github.yoep.popcorn.ui.messages.DetailsMessage;
import com.github.yoep.popcorn.ui.view.services.DetailsComponentService;
import com.github.yoep.popcorn.ui.view.services.HealthService;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import javafx.scene.control.Tooltip;
import javafx.scene.input.Clipboard;
import javafx.scene.input.ClipboardContent;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.Pane;
import lombok.extern.slf4j.Slf4j;

import java.util.List;
import java.util.Map;

/**
 * Abstract definition of a details component for showing {@link Media} information.
 *
 * @param <T> The media type of the details component.
 */
@Slf4j
public abstract class AbstractDesktopDetailsComponent<T extends Media> extends AbstractDetailsComponent<T> implements Initializable {
    protected static final String QUALITY_ACTIVE_CLASS = "active";

    protected final SubtitleService subtitleService;
    protected final SubtitlePickerService subtitlePickerService;
    protected final DetailsComponentService service;
    protected final FxLib fxLib;

    protected SubtitleInfo subtitle;
    protected String quality;

    @FXML
    Icon magnetLink;
    @FXML
    Pane qualitySelectionPane;
    @FXML
    LanguageFlagSelection languageSelection;

    //region Constructors

    protected AbstractDesktopDetailsComponent(EventPublisher eventPublisher,
                                              LocaleText localeText,
                                              HealthService healthService,
                                              SubtitleService subtitleService,
                                              SubtitlePickerService subtitlePickerService,
                                              ImageService imageService,
                                              ApplicationConfig settingsService,
                                              DetailsComponentService service,
                                              FxLib fxLib) {
        super(localeText, imageService, healthService, settingsService, eventPublisher);
        this.subtitleService = subtitleService;
        this.subtitlePickerService = subtitlePickerService;
        this.service = service;
        this.fxLib = fxLib;
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

    protected void initializeTooltips() {
        var tooltip = new Tooltip(localeText.get(DetailsMessage.MAGNET_LINK));
        instantTooltip(tooltip);
        Tooltip.install(magnetLink, tooltip);
    }

    protected void loadQualitySelection(Map<String, MediaTorrentInfo> torrents) {
        var resolutions = getVideoResolutions(torrents);
        var defaultQuality = getDefaultVideoResolution(resolutions);
        var qualities = resolutions.stream()
                .map(this::createQualityOption)
                .toList();

        // replace the quality selection with the new items
        qualitySelectionPane.getChildren().clear();
        qualitySelectionPane.getChildren().addAll(qualities);

        switchActiveQuality(defaultQuality);
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
            Platform.runLater(() -> {
                languageSelection.getItems().clear();
                languageSelection.getItems().addAll(subtitles);
                languageSelection.select(subtitleService.getDefaultOrInterfaceLanguage(subtitles));
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
                if (newValue.isNone()) {
                    subtitleService.disableSubtitle();
                } else {
                    subtitleService.updateSubtitle(newValue);
                }
            }
        };
    }

    /**
     * Reset the details component information to nothing.
     * This will allow the GC to dispose the items when the media details are no longer needed.
     */
    protected void reset() {
        this.media = null;
        this.subtitle = null;
        this.quality = null;
    }

    /**
     * Reset the language selection to the special type subtitle_none.
     */
    protected void resetLanguageSelection() {
        languageSelection.getItems().clear();
        languageSelection.getItems().add(fxLib.subtitle_none());
        languageSelection.select(0);
    }

    private void onCustomSubtitleSelected() {
        Platform.runLater(() -> {
            var subtitleInfo = subtitlePickerService.pickCustomSubtitle();

            // if a custom subtitle was picked by the user, update the subtitle with the custom subtitle
            // otherwise, the subtitle pick was cancelled and we need to reset the selected language to disabled
            subtitleInfo.ifPresentOrElse(
                    subtitleService::updateCustomSubtitle,
                    () -> languageSelection.select(fxLib.subtitle_none()));
        });
    }

    private Label createQualityOption(String quality) {
        Label label = new Label(quality);

        label.getStyleClass().add("quality");
        label.setOnMouseClicked(this::onQualityClicked);
        label.setFocusTraversable(true);

        return label;
    }

    private void onQualityClicked(MouseEvent event) {
        Label label = (Label) event.getSource();

        switchActiveQuality(label.getText());
    }

    //endregion
}
