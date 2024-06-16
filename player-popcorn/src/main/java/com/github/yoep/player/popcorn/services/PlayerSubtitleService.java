package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.popcorn.listeners.AbstractPlaybackListener;
import com.github.yoep.player.popcorn.listeners.PlaybackListener;
import com.github.yoep.player.popcorn.listeners.PlayerSubtitleListener;
import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.adapters.player.PlayRequest;
import com.github.yoep.popcorn.backend.services.AbstractListenerService;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitlePreference;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.io.FilenameUtils;

import java.util.Collections;
import java.util.List;
import java.util.Objects;
import java.util.concurrent.atomic.AtomicReference;

@Slf4j
public class PlayerSubtitleService extends AbstractListenerService<PlayerSubtitleListener> {
    private final VideoService videoService;
    private final SubtitleService subtitleService;
    private final SubtitleManagerService subtitleManagerService;
    private final FxLib fxLib;

    private final PlaybackListener listener = createListener();

    public PlayerSubtitleService(VideoService videoService, SubtitleService subtitleService, SubtitleManagerService subtitleManagerService, FxLib fxLib) {
        Objects.requireNonNull(videoService, "videoService cannot be null");
        Objects.requireNonNull(subtitleService, "subtitleService cannot be null");
        Objects.requireNonNull(subtitleManagerService, "subtitleManagerService cannot be null");
        Objects.requireNonNull(fxLib, "fxLib cannot be null");
        this.videoService = videoService;
        this.subtitleService = subtitleService;
        this.subtitleManagerService = subtitleManagerService;
        this.fxLib = fxLib;
        init();
    }

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

    private void init() {
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

            var filename = FilenameUtils.getName(request.getUrl());

            log.debug("Retrieving subtitles for \"{}\"", filename);
            subtitleService.retrieveSubtitles(filename).whenComplete((subtitles, throwable) ->
                    handleSubtitlesResponse(request.getSubtitleInfo().orElse(null), subtitles, throwable));
        }
    }

    private void handleSubtitlesResponse(SubtitleInfo requestSubtitle, List<SubtitleInfo> subtitles, Throwable throwable) {
        if (throwable == null) {
            log.trace("Available subtitles have been retrieved");
            try (var preference = subtitleService.preference()) {
                var tag = preference.getTag();
                var activeSubtitle = new AtomicReference<>(subtitleService.none());

                if (tag != SubtitlePreference.Tag.DISABLED) {
                    if (requestSubtitle == null || requestSubtitle.isNone()) {
                        log.trace("Selecting a new default subtitle to enable during playback");
                        activeSubtitle.set(subtitleService.getDefaultOrInterfaceLanguage(subtitles));
                        subtitleService.updatePreferredLanguage(activeSubtitle.get().getLanguage());
                    } else {
                        log.trace("Using request subtitle {}", requestSubtitle);
                        activeSubtitle.set(requestSubtitle);
                    }
                }

                invokeListeners(e -> e.onAvailableSubtitlesChanged(subtitles, activeSubtitle.get()));
            }
        } else {
            log.error("Failed to retrieve subtitles, {}", throwable.getMessage(), throwable);
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
