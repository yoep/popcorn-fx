package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.ui.media.providers.models.Media;
import com.github.yoep.popcorn.ui.settings.SettingsService;
import com.github.yoep.popcorn.ui.view.controllers.common.components.AbstractDetailsComponent;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import com.github.yoep.torrent.adapter.TorrentService;
import lombok.extern.slf4j.Slf4j;

@Slf4j
public abstract class AbstractTvDetailsComponent<T extends Media> extends AbstractDetailsComponent<T> {
    protected AbstractTvDetailsComponent(LocaleText localeText, ImageService imageService, TorrentService torrentService, SettingsService settingsService) {
        super(localeText, imageService, torrentService, settingsService);
    }
}
