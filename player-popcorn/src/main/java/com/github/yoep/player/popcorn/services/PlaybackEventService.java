package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.popcorn.controllers.components.PlayerControlsComponent;
import com.github.yoep.player.popcorn.controllers.components.PlayerHeaderComponent;
import com.github.yoep.player.popcorn.controllers.sections.PopcornPlayerSectionController;
import com.github.yoep.player.popcorn.listeners.PlaybackListener;
import com.github.yoep.player.popcorn.player.PopcornPlayer;
import com.github.yoep.popcorn.backend.adapters.player.PlayRequest;
import com.github.yoep.popcorn.backend.adapters.player.listeners.PlayerListener;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
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

/**
 * The playback event service is for handling the events triggered by the playback.
 */
@Slf4j
@Service
@RequiredArgsConstructor
public class PlaybackEventService implements PlaybackListener, PlayerListener {
    private final PopcornPlayer player;
    private final VideoService videoService;
    private final SubtitleService subtitleService;
    private final PopcornPlayerSectionController playerSection;
    private final PlayerHeaderComponent playerHeader;
    private final PlayerControlsComponent playerControls;

    private SubtitleInfo subtitle;

    //region PlaybackListener

    @Override
    public void onPlay(PlayRequest request) {
        handleSectionOnPlay();
        handleHeaderOnPlay(request);
        handleControlsOnPlay(request);
    }

    @Override
    public void onResume() {
        // no-op
    }

    @Override
    public void onPause() {
        // no-op
    }

    @Override
    public void onSeek(long time) {
        // no-op
    }

    @Override
    public void onVolume(int volume) {
        // no-op
    }

    @Override
    public void onStop() {
        playerSection.reset();
        playerControls.reset();
    }

    //endregion

    //region PlayerListener

    @Override
    public void onDurationChanged(long newDuration) {
        onPlayerDurationChanged(newDuration);
    }

    @Override
    public void onTimeChanged(long newTime) {
        onPlayerTimeChanged(newTime);
    }

    @Override
    public void onStateChanged(PlayerState newState) {
        onPlayerStateChanged(newState);
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    void init() {
        initializeListeners();
    }

    private void initializeListeners() {
        player.addListener(this);
        videoService.addListener(this);

    }

    //endregion

    //region Functions

    private void handleSectionOnPlay() {
        playerSection.reset();
    }

    private void handleHeaderOnPlay(PlayRequest request) {
        playerHeader.updateTitle(request.getTitle().orElse(null));
        playerHeader.updateQuality(request.getQuality().orElse(null));
    }

    private void handleControlsOnPlay(PlayRequest request) {
        playerControls.updateSubtitleVisibility(request.isSubtitlesEnabled());

        // check if the activity contains media information
        if (request instanceof MediaPlayRequest) {
            var mediaActivity = (MediaPlayRequest) request;
            onPlayMedia(mediaActivity);
            return;
        }

        if (request.isSubtitlesEnabled()) {
            // set the default subtitle to "none" when loading
            var defaultSubtitle = SubtitleInfo.none();
            playerControls.updateAvailableSubtitles(Collections.singletonList(defaultSubtitle), defaultSubtitle);

            String filename = FilenameUtils.getName(request.getUrl());

            log.debug("Retrieving subtitles for \"{}\"", filename);
            subtitleService.retrieveSubtitles(filename).whenComplete(this::handleSubtitlesResponse);
        }
    }

    private void onPlayMedia(MediaPlayRequest activity) {
        var media = activity.getMedia();

        // set the subtitle for the playback
        this.subtitle = activity.getSubtitle()
                .orElse(SubtitleInfo.none());

        if (media instanceof Movie) {
            var movie = (Movie) activity.getMedia();
            subtitleService.retrieveSubtitles(movie).whenComplete(this::handleSubtitlesResponse);
        } else if (media instanceof Episode) {
            Episode episode = (Episode) activity.getMedia();

            subtitleService.retrieveSubtitles(episode.getShow(), episode).whenComplete(this::handleSubtitlesResponse);
        } else {
            log.error("Failed to retrieve subtitles, invalid media type");
        }
    }

    private void onPlayerDurationChanged(Long duration) {
        playerControls.updateDuration(duration);
    }

    private void onPlayerTimeChanged(Long time) {
        playerControls.updateTime(time);
        playerSection.updateTime(time);
    }

    private void onPlayerStateChanged(PlayerState newState) {
        playerControls.updatePlaybackState(newState != PlayerState.PAUSED);
        playerSection.updatePlaybackState(newState != PlayerState.PAUSED);
    }

    private void handleSubtitlesResponse(final List<SubtitleInfo> subtitles, Throwable throwable) {
        if (throwable == null) {
            var subtitle = this.subtitle != null ? this.subtitle : subtitleService.getDefault(subtitles);
            playerControls.updateAvailableSubtitles(subtitles, subtitle);
        } else {
            log.error("Failed to retrieve subtitles, " + throwable.getMessage(), throwable);
        }
    }

    //endregion
}
