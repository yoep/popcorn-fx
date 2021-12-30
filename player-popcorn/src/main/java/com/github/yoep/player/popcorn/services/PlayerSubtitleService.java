package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.popcorn.listeners.AbstractPlaybackListener;
import com.github.yoep.player.popcorn.listeners.PlaybackListener;
import com.github.yoep.player.popcorn.listeners.PlayerSubtitleListener;
import com.github.yoep.popcorn.backend.adapters.player.PlayRequest;
import com.github.yoep.popcorn.backend.media.providers.models.Episode;
import com.github.yoep.popcorn.backend.media.providers.models.Movie;
import com.github.yoep.popcorn.backend.player.model.MediaPlayRequest;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.backend.subtitles.models.SubtitleInfo;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.io.FilenameUtils;
import org.springframework.stereotype.Service;

import javax.annotation.PostConstruct;
import java.util.Collections;
import java.util.List;

@Slf4j
@Service
@RequiredArgsConstructor
public class PlayerSubtitleService extends AbstractListenerService<PlayerSubtitleListener> {
    private final VideoService videoService;
    private final SubtitleService subtitleService;
    private final SubtitleManagerService subtitleManagerService;

    private final PlaybackListener listener = createListener();

    private SubtitleInfo subtitle;

    //region Methods

    public void updateSubtitleSizeWithSizeOffset(int pixelChange) {
        subtitleManagerService.setSubtitleOffset(pixelChange);
    }

    public void updateActiveSubtitle(SubtitleInfo subtitleInfo) {
        subtitleManagerService.setSubtitle(subtitleInfo);
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    void init() {
        subtitleService.activeSubtitleProperty().addListener((observableValue, subtitle, newSubtitle) ->
                invokeListeners(e -> e.onActiveSubtitleChanged(newSubtitle.getSubtitleInfo().orElse(SubtitleInfo.none()))));
        videoService.addListener(listener);
    }

    //endregion

    //region Functions

    private void onPlayRequest(PlayRequest request) {
        if (request.isSubtitlesEnabled()) {
            // set the default subtitle to "none" when loading
            var defaultSubtitle = SubtitleInfo.none();
            invokeListeners(e -> e.onAvailableSubtitlesChanged(Collections.singletonList(defaultSubtitle), defaultSubtitle));

            String filename = FilenameUtils.getName(request.getUrl());

            log.debug("Retrieving subtitles for \"{}\"", filename);
            subtitleService.retrieveSubtitles(filename).whenComplete(this::handleSubtitlesResponse);
        }
    }

    private void onMediaPlayRequest(MediaPlayRequest request) {
        // set the subtitle for the playback
        this.subtitle = request.getSubtitle()
                .orElse(SubtitleInfo.none());

        var media = request.getMedia();

        if (media instanceof Movie) {
            var movie = (Movie) request.getMedia();
            log.trace("Retrieving movie subtitles for {}", movie);
            subtitleService.retrieveSubtitles(movie).whenComplete(this::handleSubtitlesResponse);
        } else if (media instanceof Episode) {
            Episode episode = (Episode) request.getMedia();

            log.trace("Retrieving episode subtitles for {}", episode);
            subtitleService.retrieveSubtitles(episode.getShow(), episode).whenComplete(this::handleSubtitlesResponse);
        } else {
            log.error("Failed to retrieve subtitles, invalid media type {}", media.getClass().getSimpleName());
        }
    }

    private void handleSubtitlesResponse(final List<SubtitleInfo> subtitles, Throwable throwable) {
        if (throwable == null) {
            log.trace("Available subtitles have been retrieved");
            var subtitle = this.subtitle != null ? this.subtitle : subtitleService.getDefault(subtitles);

            invokeListeners(e -> e.onAvailableSubtitlesChanged(subtitles, subtitle));
        } else {
            log.error("Failed to retrieve subtitles, " + throwable.getMessage(), throwable);
        }
    }

    private PlaybackListener createListener() {
        return new AbstractPlaybackListener() {
            @Override
            public void onPlay(PlayRequest request) {
                if (request instanceof MediaPlayRequest) {
                    onMediaPlayRequest((MediaPlayRequest) request);
                } else {
                    onPlayRequest(request);
                }
            }

            @Override
            public void onStop() {
                subtitle = null;
            }
        };
    }

    //endregion
}
