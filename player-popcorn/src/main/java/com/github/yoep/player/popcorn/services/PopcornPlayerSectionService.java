package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.popcorn.listeners.AbstractPlayerListener;
import com.github.yoep.player.popcorn.listeners.PopcornPlayerSectionListener;
import com.github.yoep.player.popcorn.listeners.SubtitleListener;
import com.github.yoep.player.popcorn.player.PopcornPlayer;
import com.github.yoep.popcorn.backend.adapters.player.listeners.PlayerListener;
import com.github.yoep.popcorn.backend.adapters.screen.ScreenService;
import com.github.yoep.popcorn.backend.adapters.video.VideoPlayback;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationSettings;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Player;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Subtitle;
import com.github.yoep.popcorn.backend.services.AbstractListenerService;
import com.github.yoep.popcorn.backend.settings.AbstractApplicationSettingsEventListener;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.settings.ApplicationSettingsEventListener;
import com.github.yoep.popcorn.backend.subtitles.ISubtitle;
import com.github.yoep.popcorn.backend.subtitles.ISubtitleInfo;
import com.github.yoep.popcorn.backend.subtitles.SubtitleWrapper;
import lombok.extern.slf4j.Slf4j;

import java.util.List;

@Slf4j
public class PopcornPlayerSectionService extends AbstractListenerService<PopcornPlayerSectionListener> {
    private final PopcornPlayer player;
    private final ScreenService screenService;
    private final ApplicationConfig applicationConfig;
    private final SubtitleManagerService subtitleManagerService;
    private final VideoService videoService;

    private final PlayerListener playerListener = createPlayerListener();

    public PopcornPlayerSectionService(PopcornPlayer player, ScreenService screenService, ApplicationConfig applicationConfig, SubtitleManagerService subtitleManagerService, VideoService videoService) {
        this.player = player;
        this.screenService = screenService;
        this.applicationConfig = applicationConfig;
        this.subtitleManagerService = subtitleManagerService;
        this.videoService = videoService;
        init();
    }

    //region Methods

    public boolean isNativeSubtitlePlaybackSupported() {
        return videoService.getVideoPlayer()
                .map(VideoPlayback::supportsNativeSubtitleFile)
                .orElse(false);
    }

    public void togglePlayerPlaybackState() {
        if (player.getState() == Player.State.PAUSED) {
            player.resume();
        } else {
            player.pause();
        }
    }

    public void toggleFullscreen() {
        screenService.toggleFullscreen();
    }

    public void videoTimeOffset(int offset) {
        player.seek(player.getTime() + offset);
    }

    public void onVolumeScroll(double volumeDelta) {
        log.trace("Updating the player volume with an offset of {}", volumeDelta);
        var currentVolume = player.getVolume();
        var newVolume = (int) (currentVolume + volumeDelta);

        // check if the new value is between 100 or 0
        if (newVolume > 100) {
            newVolume = 100;
        } else if (newVolume < 0) {
            newVolume = 0;
        }

        log.trace("Updating the player volume to {}", newVolume);
        player.volume(newVolume);
    }

    public void provideSubtitleValues() {
        applicationConfig.getSettings().thenApply(ApplicationSettings::getSubtitleSettings).whenComplete((settings, throwable) -> {
            if (throwable == null) {
                invokeListeners(e -> e.onSubtitleFamilyChanged(settings.getFontFamily().name()));
                invokeListeners(e -> e.onSubtitleFontWeightChanged(settings.getBold()));
                invokeListeners(e -> e.onSubtitleSizeChanged(settings.getFontSize()));
                invokeListeners(e -> e.onSubtitleDecorationChanged(settings.getDecoration()));
            } else {
                log.error("Failed to retrieve settings", throwable);
            }
        });
    }

    //endregion

    //region PostConstruct

    private void init() {
        initializeListeners();
    }

    private void initializeListeners() {
        player.addListener(playerListener);
        applicationConfig.addListener(new AbstractApplicationSettingsEventListener() {
            @Override
            public void onSubtitleSettingsChanged(ApplicationSettings.SubtitleSettings settings) {
                PopcornPlayerSectionService.this.onSubtitleSettingsChanged(settings);
            }
        });
        videoService.videoPlayerProperty().addListener((observableValue, videoPlayer, newVideoPlayer) -> onVideoViewChanged(newVideoPlayer));
        subtitleManagerService.subtitleSizeProperty().addListener((observableValue, number, newSize) -> onSubtitleSizeChanged(newSize));
        subtitleManagerService.registerListener(new SubtitleListener() {
            @Override
            public void onSubtitleChanged(ISubtitle newSubtitle) {
                onActiveSubtitleChanged(newSubtitle);
            }

            @Override
            public void onSubtitleDisabled() {
                invokeListeners(PopcornPlayerSectionListener::onSubtitleDisabled);
            }

            @Override
            public void onAvailableSubtitlesChanged(List<ISubtitleInfo> subtitles) {
                // no-op
            }
        });
    }

    //endregion

    private void onActiveSubtitleChanged(ISubtitle newSubtitle) {
        if (newSubtitle == null)
            return;

        invokeListeners(e -> e.onSubtitleChanged(newSubtitle));
    }

    private void onSubtitleSizeChanged(Number newSize) {
        invokeListeners(e -> e.onSubtitleSizeChanged(newSize.intValue()));
    }

    private void onSubtitleSettingsChanged(ApplicationSettings.SubtitleSettings settings) {
        invokeListeners(e -> e.onSubtitleFamilyChanged(settings.getFontFamily().name()));
        invokeListeners(e -> e.onSubtitleSizeChanged(settings.getFontSize()));
        invokeListeners(e -> e.onSubtitleFontWeightChanged(settings.getBold()));
        invokeListeners(e -> e.onSubtitleDecorationChanged(settings.getDecoration()));
    }

    private void onPlayerTimeChanged(long newTime) {
        invokeListeners(e -> e.onPlayerTimeChanged(newTime));
    }

    private void onPlayerStateChanged(Player.State newState) {
        log.trace("Video player entered state {}", newState);
        invokeListeners(e -> e.onPlayerStateChanged(newState));
    }

    private void onPlayerVolumeChanged(int volume) {
        invokeListeners(e -> e.onVolumeChanged(volume));
    }

    private void onVideoViewChanged(VideoPlayback newVideoPlayback) {
        invokeListeners(e -> e.onVideoViewChanged(newVideoPlayback.getVideoSurface()));
    }

    private PlayerListener createPlayerListener() {
        return new AbstractPlayerListener() {
            @Override
            public void onTimeChanged(long newTime) {
                onPlayerTimeChanged(newTime);
            }

            @Override
            public void onStateChanged(Player.State newState) {
                onPlayerStateChanged(newState);
            }

            @Override
            public void onVolumeChanged(int volume) {
                onPlayerVolumeChanged(volume);
            }
        };
    }
}
