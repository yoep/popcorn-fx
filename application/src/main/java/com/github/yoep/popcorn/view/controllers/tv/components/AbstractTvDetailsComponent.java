package com.github.yoep.popcorn.view.controllers.tv.components;

import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.settings.SettingsService;
import com.github.yoep.popcorn.torrent.TorrentService;
import com.github.yoep.popcorn.view.controllers.common.components.AbstractDetailsComponent;
import com.github.yoep.popcorn.view.services.ImageService;
import lombok.extern.slf4j.Slf4j;

@Slf4j
public abstract class AbstractTvDetailsComponent<T extends Media> extends AbstractDetailsComponent<T> {
    protected AbstractTvDetailsComponent(ImageService imageService, TorrentService torrentService, SettingsService settingsService) {
        super(imageService, torrentService, settingsService);
    }
}
