package com.github.yoep.popcorn.ui.player;

import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.adapters.screen.ScreenService;
import com.github.yoep.popcorn.backend.events.PlayMediaEvent;
import com.github.yoep.popcorn.backend.events.PlayVideoEvent;
import com.github.yoep.popcorn.backend.events.PlayVideoTorrentEvent;
import com.github.yoep.popcorn.backend.media.resume.AutoResumeService;
import com.github.yoep.popcorn.backend.player.model.MediaPlayRequest;
import com.github.yoep.popcorn.backend.player.model.SimplePlayRequest;
import com.github.yoep.popcorn.backend.player.model.StreamPlayRequest;
import com.github.yoep.popcorn.backend.settings.SettingsService;
import com.github.yoep.popcorn.backend.subtitles.Subtitle;
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
    private final ScreenService screenService;
    private final SettingsService settingsService;

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
        } else if (event instanceof PlayVideoTorrentEvent) {
            playStreamVideo((PlayVideoTorrentEvent) event, player);
        } else {
            playSimpleVideo(event, player);
        }

        // check if the user prefers to start the video playback in fullscreen mode
        fullscreenVideo();
    }

    private void playMediaVideo(PlayMediaEvent event, Player player) {
        var filename = FilenameUtils.getName(event.getUrl());

        player.play(MediaPlayRequest.mediaBuilder()
                .url(event.getUrl())
                .title(event.getTitle())
                .thumb(event.getThumbnail())
                .quality(event.getQuality())
                .media(event.getMedia())
                .subtitle(event.getSubtitle()
                        .flatMap(Subtitle::getSubtitleInfo)
                        .orElse(null))
                .autoResumeTimestamp(autoResumeService.getResumeTimestamp(event.getMedia().getId(), filename).orElse(null))
                .torrentStream(event.getTorrentStream())
                .build());
    }

    private void playStreamVideo(PlayVideoTorrentEvent event, Player player) {
        var filename = FilenameUtils.getName(event.getUrl());

        player.play(StreamPlayRequest.streamBuilder()
                .url(event.getUrl())
                .title(event.getTitle())
                .thumb(event.getThumbnail())
                .autoResumeTimestamp(autoResumeService.getResumeTimestamp(filename).orElse(null))
                .torrentStream(event.getTorrentStream())
                .build());
    }

    private void playSimpleVideo(PlayVideoEvent event, Player player) {
        var filename = FilenameUtils.getName(event.getUrl());

        player.play(SimplePlayRequest.builder()
                .url(event.getUrl())
                .title(event.getTitle())
                .thumb(event.getThumbnail())
                .autoResumeTimestamp(autoResumeService.getResumeTimestamp(filename).orElse(null))
                .build());
    }

    private void fullscreenVideo() {
        var playbackSettings = settingsService.getSettings().getPlaybackSettings();

        if (playbackSettings.isFullscreen()) {
            screenService.fullscreen(true);
        }
    }

    //endregion
}
