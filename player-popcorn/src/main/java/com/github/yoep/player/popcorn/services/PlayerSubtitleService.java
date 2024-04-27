package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.popcorn.listeners.AbstractPlaybackListener;
import com.github.yoep.player.popcorn.listeners.PlaybackListener;
import com.github.yoep.player.popcorn.listeners.PlayerSubtitleListener;
import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.adapters.player.PlayRequest;
import com.github.yoep.popcorn.backend.services.AbstractListenerService;
import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleLanguage;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.io.FilenameUtils;

import javax.annotation.PostConstruct;
import java.util.Collections;
import java.util.List;

@Slf4j

@RequiredArgsConstructor
public class PlayerSubtitleService extends AbstractListenerService<PlayerSubtitleListener> {
    private final VideoService videoService;
    private final SubtitleService subtitleService;
    private final SubtitleManagerService subtitleManagerService;
    private final FxLib fxLib;

    private final PlaybackListener listener = createListener();

    //region Methods

    public void updateSubtitleSizeWithSizeOffset(int pixelChange) {
        subtitleManagerService.setSubtitleSize(subtitleManagerService.getSubtitleSize() + pixelChange);
    }

    public void updateActiveSubtitle(SubtitleInfo subtitleInfo) {
        subtitleManagerService.updateSubtitle(subtitleInfo);
    }

    public SubtitleInfo[] defaultSubtitles() {
        try (var none = subtitleService.none()) {
            try (var custom = subtitleService.custom()) {
                return new SubtitleInfo[]{none, custom};
            }
        }
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    void init() {
        videoService.addListener(listener);
    }

    //endregion

    //region Functions

    private void onPlayRequest(PlayRequest request) {
        if (request.isSubtitlesEnabled()) {
            // set the default subtitle to "none" when loading
            try (var defaultSubtitle = fxLib.subtitle_none()) {
                invokeListeners(e -> e.onAvailableSubtitlesChanged(Collections.singletonList(defaultSubtitle), defaultSubtitle));
            }

            String filename = FilenameUtils.getName(request.getUrl());

            log.debug("Retrieving subtitles for \"{}\"", filename);
            subtitleService.retrieveSubtitles(filename).whenComplete(this::handleSubtitlesResponse);
        }
    }

    private void handleSubtitlesResponse(final List<SubtitleInfo> subtitles, Throwable throwable) {
        if (throwable == null) {
            log.trace("Available subtitles have been retrieved");
            if (!subtitleService.isDisabled() && subtitleService.preferredSubtitleLanguage() == SubtitleLanguage.NONE) {
                log.trace("Selecting a new default subtitle to enable during playback");
                subtitleService.updateSubtitle(subtitleService.getDefaultOrInterfaceLanguage(subtitles));
            }

            invokeListeners(e -> e.onAvailableSubtitlesChanged(subtitles,
                    subtitleService.preferredSubtitle().orElseGet(fxLib::subtitle_none)));
        } else {
            log.error("Failed to retrieve subtitles, " + throwable.getMessage(), throwable);
        }
    }

    private PlaybackListener createListener() {
        return new AbstractPlaybackListener() {
            @Override
            public void onPlay(PlayRequest request) {
                onPlayRequest(request);
            }

            @Override
            public void onStop() {
                // no-op
            }
        };
    }

    //endregion
}
