package com.github.yoep.popcorn.ui.player;

import com.github.yoep.player.adapter.Player;
import com.github.yoep.player.adapter.PlayerManagerService;
import com.github.yoep.popcorn.ui.events.PlayMediaEvent;
import com.github.yoep.popcorn.ui.events.PlayVideoEvent;
import com.github.yoep.popcorn.ui.media.resume.AutoResumeService;
import com.github.yoep.popcorn.ui.player.model.MediaPlayRequest;
import com.github.yoep.popcorn.ui.player.model.SimplePlayRequest;
import com.github.yoep.popcorn.ui.settings.SettingsService;
import com.github.yoep.popcorn.ui.subtitles.SubtitleService;
import com.github.yoep.popcorn.ui.view.services.FullscreenService;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.io.FilenameUtils;
import org.springframework.context.event.EventListener;
import org.springframework.stereotype.Service;

@Slf4j
@Service
@RequiredArgsConstructor
public class PlayerPlayService {
    private final PlayerManagerService playerManagerService;
    private final AutoResumeService autoResumeService;
    private final FullscreenService fullscreenService;
    private final SettingsService settingsService;
    private final SubtitleService subtitleService;

    //region Methods

    @EventListener
    public void onPlayVideo(PlayVideoEvent event) {
        // retrieve the current active player
        // and use it to play the video playback on
        playerManagerService.getActivePlayer().ifPresentOrElse(
                player -> playVideo(event, player),
                () -> log.error("Failed to play video {}, there is no active player", event.getUrl()) // this should never occur
        );
    }

    //endregion

    //region Functions

    private void playVideo(PlayVideoEvent event, Player player) {
        log.debug("Starting playback of {} in player {}", event.getUrl(), player.getName());
        if (event instanceof PlayMediaEvent) {
            playMediaVideo((PlayMediaEvent) event, player);
        } else {
            playSimpleVideo(event, player);
        }

        // check if the user prefers to start the video playback in fullscreen mode
        fullscreenVideo();

        // check if a known resume timestamp is known for the current play event
        // if so, we'll try to auto resume the last known timestamp back in the player
        autoResumeVideo(event, player);
    }

    private void playMediaVideo(PlayMediaEvent event, Player player) {
        player.play(MediaPlayRequest.mediaBuilder()
                .url(event.getUrl())
                .title(event.getTitle())
                .thumb(event.getThumbnail())
                .quality(event.getQuality())
                .subtitle(event.getSubtitle().orElse(null))
                .build());
    }

    private void playSimpleVideo(PlayVideoEvent event, Player player) {
        player.play(SimplePlayRequest.builder()
                .url(event.getUrl())
                .title(event.getTitle())
                .thumb(event.getThumbnail())
                .build());
    }

    private void fullscreenVideo() {
        var playbackSettings = settingsService.getSettings().getPlaybackSettings();

        if (playbackSettings.isFullscreen()) {
            fullscreenService.fullscreen(true);
        }
    }

    private void autoResumeVideo(PlayVideoEvent event, Player player) {
        var filename = FilenameUtils.getName(event.getUrl());

        if (event instanceof PlayMediaEvent) {
            var mediaEvent = (PlayMediaEvent) event;
            var media = mediaEvent.getMedia();

            autoResumeService.getResumeTimestamp(media.getId(), filename)
                    .ifPresent(player::seek);
        } else {
            autoResumeService.getResumeTimestamp(filename)
                    .ifPresent(player::seek);
        }
    }

    //endregion
}
