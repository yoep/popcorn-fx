package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.popcorn.listeners.AbstractPlaybackListener;
import com.github.yoep.player.popcorn.listeners.PlaybackListener;
import com.github.yoep.player.popcorn.listeners.PlayerSubtitleListener;
import com.github.yoep.popcorn.backend.adapters.player.PlayRequest;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Subtitle;
import com.github.yoep.popcorn.backend.services.AbstractListenerService;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.io.FilenameUtils;

import java.util.List;
import java.util.Objects;

@Slf4j
public class PlayerSubtitleService extends AbstractListenerService<PlayerSubtitleListener> {
    private final VideoService videoService;
    private final SubtitleService subtitleService;
    private final SubtitleManagerService subtitleManagerService;

    private final PlaybackListener listener = createListener();

    public PlayerSubtitleService(VideoService videoService, SubtitleService subtitleService, SubtitleManagerService subtitleManagerService) {
        Objects.requireNonNull(videoService, "videoService cannot be null");
        Objects.requireNonNull(subtitleService, "subtitleService cannot be null");
        Objects.requireNonNull(subtitleManagerService, "subtitleManagerService cannot be null");
        this.videoService = videoService;
        this.subtitleService = subtitleService;
        this.subtitleManagerService = subtitleManagerService;
        init();
    }

    //region Methods

    public void updateSubtitleSizeWithSizeOffset(int pixelChange) {
        subtitleManagerService.setSubtitleSize(subtitleManagerService.getSubtitleSize() + pixelChange);
    }

    public void updateActiveSubtitle(Subtitle.Info subtitleInfo) {
        subtitleManagerService.updateSubtitle(subtitleInfo);
    }

    public Subtitle.Info[] defaultSubtitles() {
        try {
            var none = subtitleService.none().get();
            var custom = subtitleService.custom().get();

            return new Subtitle.Info[]{none, custom};
        } catch (Exception e) {
            throw new RuntimeException(e);
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
            var defaultSubtitle = subtitleService.none();
//            invokeListeners(e -> e.onAvailableSubtitlesChanged(Collections.singletonList(defaultSubtitle), defaultSubtitle));

            var filename = FilenameUtils.getName(request.getUrl());

            log.debug("Retrieving subtitles for \"{}\"", filename);
//            subtitleService.retrieveSubtitles(filename).whenComplete((subtitles, throwable) ->
//                    handleSubtitlesResponse(request.getSubtitleInfo().orElse(null), subtitles, throwable));
        }
    }

    private void handleSubtitlesResponse(Subtitle.Info requestSubtitle, List<Subtitle.Info> subtitles, Throwable throwable) {
        if (throwable == null) {
//            log.trace("Available subtitles have been retrieved");
//            var preference = subtitleService.preference();
//            var tag = preference.tag();
//            var activeSubtitle = new AtomicReference<>(subtitleService.none());
//
//            if (tag != SubtitlePreferenceTag.DISABLED) {
//                if (requestSubtitle == null || requestSubtitle.isNone()) {
//                    log.trace("Selecting a new default subtitle to enable during playback");
//                    activeSubtitle.set(subtitleService.getDefaultOrInterfaceLanguage(subtitles));
//                    subtitleService.updatePreferredLanguage(activeSubtitle.get().language());
//                } else {
//                    log.trace("Using request subtitle {}", requestSubtitle);
//                    activeSubtitle.set(requestSubtitle);
//                }
//            }
//
//            invokeListeners(e -> e.onAvailableSubtitlesChanged(subtitles, activeSubtitle.get()));
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
