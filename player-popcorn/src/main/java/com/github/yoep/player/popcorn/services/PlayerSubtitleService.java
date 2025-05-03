package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.popcorn.listeners.AbstractPlaybackListener;
import com.github.yoep.player.popcorn.listeners.PlaybackListener;
import com.github.yoep.player.popcorn.listeners.PlayerSubtitleListener;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Player;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.SubtitlePreference;
import com.github.yoep.popcorn.backend.services.AbstractListenerService;
import com.github.yoep.popcorn.backend.subtitles.ISubtitleInfo;
import com.github.yoep.popcorn.backend.subtitles.SubtitleInfoWrapper;
import com.github.yoep.popcorn.backend.subtitles.ISubtitleService;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.io.FilenameUtils;

import java.util.List;
import java.util.Objects;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;

@Slf4j
public class PlayerSubtitleService extends AbstractListenerService<PlayerSubtitleListener> {
    private final VideoService videoService;
    private final ISubtitleService subtitleService;
    private final SubtitleManagerService subtitleManagerService;

    private final PlaybackListener listener = createListener();
    private ISubtitleInfo subtitleNone;

    public PlayerSubtitleService(VideoService videoService, ISubtitleService subtitleService, SubtitleManagerService subtitleManagerService) {
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

    public void updateActiveSubtitle(ISubtitleInfo subtitleInfo) {
        subtitleManagerService.updateSubtitle(subtitleInfo);
    }

    public CompletableFuture<List<ISubtitleInfo>> defaultSubtitles() {
        return subtitleService.defaultSubtitles();
    }

    //endregion

    //region PostConstruct

    private void init() {
        videoService.addListener(listener);
        subtitleService.defaultSubtitles()
                .thenApply(List::getFirst)
                .thenAccept(subtitle -> subtitleNone = subtitle);
    }

    //endregion

    //region Functions

    private void onPlayRequest(Player.PlayRequest request) {
        if (request.getSubtitle().getEnabled()) {
            // set the default subtitle to "none" when loading
            subtitleService.defaultSubtitles()
                    .thenAccept(subtitles ->
                            invokeListeners(e -> e.onAvailableSubtitlesChanged(subtitles, subtitles.getFirst())));

            // TODO: update request so that the media item is present again and the subtitles can be retrieved based on ID
            var filename = FilenameUtils.getName(request.getUrl());

            log.debug("Retrieving subtitles for \"{}\"", filename);
            subtitleService.retrieveSubtitles(filename).thenAccept(subtitles ->
                    handleSubtitlesResponse(Optional.ofNullable(request.getSubtitle().getInfo())
                            .filter(e -> request.getSubtitle().hasInfo())
                            .map(SubtitleInfoWrapper::new)
                            .orElse(null), subtitles));
        }
    }

    private void handleSubtitlesResponse(ISubtitleInfo requestSubtitle, List<ISubtitleInfo> subtitles) {
        log.trace("Available subtitles have been retrieved");
        subtitleService.preference().thenAccept(preference -> {
            if (preference.getPreference() == SubtitlePreference.Preference.LANGUAGE) {
                if (requestSubtitle == null || requestSubtitle.isNone()) {
                    log.trace("Selecting a new default subtitle to enable during playback");
                    subtitleService.getDefaultOrInterfaceLanguage(subtitles)
                            .thenAccept(subtitle -> {
                                subtitleService.updatePreferredLanguage(subtitle.getLanguage());
                                invokeListeners(e -> e.onAvailableSubtitlesChanged(subtitles, subtitle));
                            });
                } else {
                    log.trace("Using request subtitle {}", requestSubtitle);
                    invokeListeners(e -> e.onAvailableSubtitlesChanged(subtitles, subtitleNone));
                }
            }
        });
    }

    private PlaybackListener createListener() {
        return new AbstractPlaybackListener() {
            @Override
            public void onPlay(Player.PlayRequest request) {
                onPlayRequest(request);
            }

            @Override
            public void onStop() {
                invokeListeners(e -> e.onActiveSubtitleChanged(subtitleNone));
            }
        };
    }

    //endregion
}
