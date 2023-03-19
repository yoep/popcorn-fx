package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.providers.models.MediaTorrentInfo;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.subtitles.SubtitlePickerService;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.ui.events.OpenMagnetLinkEvent;
import com.github.yoep.popcorn.ui.events.SuccessNotificationEvent;
import com.github.yoep.popcorn.ui.messages.DetailsMessage;
import com.github.yoep.popcorn.ui.view.ViewHelper;
import com.github.yoep.popcorn.ui.view.services.DetailsComponentService;
import com.github.yoep.popcorn.ui.view.services.HealthService;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Tooltip;
import javafx.scene.input.Clipboard;
import javafx.scene.input.ClipboardContent;
import lombok.extern.slf4j.Slf4j;

/**
 * Abstract definition of a details component for showing {@link Media} information.
 *
 * @param <T> The media type of the details component.
 */
@Slf4j
public abstract class AbstractDesktopDetailsComponent<T extends Media> extends AbstractDetailsComponent<T> implements Initializable {

    protected final SubtitleService subtitleService;
    protected final SubtitlePickerService subtitlePickerService;
    protected final DetailsComponentService service;

    @FXML
    Icon magnetLink;

    //region Constructors

    protected AbstractDesktopDetailsComponent(EventPublisher eventPublisher,
                                              LocaleText localeText,
                                              HealthService healthService,
                                              SubtitleService subtitleService,
                                              SubtitlePickerService subtitlePickerService,
                                              ImageService imageService,
                                              ApplicationConfig settingsService,
                                              DetailsComponentService service) {
        super(localeText, imageService, healthService, settingsService, eventPublisher);
        this.subtitleService = subtitleService;
        this.subtitlePickerService = subtitlePickerService;
        this.service = service;
    }

    //endregion

    //region AbstractDetailsComponent

    @Override
    protected void loadStars() {
        super.loadStars();

        var tooltip = new Tooltip(getRatingText());
        ViewHelper.instantTooltip(tooltip);
        Tooltip.install(ratingStars, tooltip);
    }

    //endregion

    //region Functions

    protected void initializeTooltips() {
        var tooltip = new Tooltip(localeText.get(DetailsMessage.MAGNET_LINK));
        ViewHelper.instantTooltip(tooltip);
        Tooltip.install(magnetLink, tooltip);
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
     * Reset the details component information to nothing.
     * This will allow the GC to dispose the items when the media details are no longer needed.
     */
    protected void reset() {
        this.media = null;
    }

    //endregion
}
