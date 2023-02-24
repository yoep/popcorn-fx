package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.popcorn.listeners.AbstractPlayerListener;
import com.github.yoep.player.popcorn.listeners.PopcornPlayerSectionListener;
import com.github.yoep.player.popcorn.listeners.SubtitleListener;
import com.github.yoep.player.popcorn.player.PopcornPlayer;
import com.github.yoep.popcorn.backend.adapters.player.listeners.PlayerListener;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import com.github.yoep.popcorn.backend.adapters.screen.ScreenService;
import com.github.yoep.popcorn.backend.adapters.video.VideoPlayback;
import com.github.yoep.popcorn.backend.services.AbstractListenerService;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.settings.ApplicationConfigEvent;
import com.github.yoep.popcorn.backend.subtitles.Subtitle;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

import javax.annotation.PostConstruct;

@Slf4j
@Service
@RequiredArgsConstructor
public class PopcornPlayerSectionService extends AbstractListenerService<PopcornPlayerSectionListener> {
    private final PopcornPlayer player;
    private final ScreenService screenService;
    private final ApplicationConfig applicationConfig;
    private final SubtitleManagerService subtitleManagerService;
    private final VideoService videoService;

    private final PlayerListener playerListener = createPlayerListener();

    //region Methods

    public boolean isNativeSubtitlePlaybackSupported() {
        return videoService.getVideoPlayer()
                .map(VideoPlayback::supportsNativeSubtitleFile)
                .orElse(false);
    }

    public void togglePlayerPlaybackState() {
        if (player.getState() == PlayerState.PAUSED) {
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
        var subtitleSettings = applicationConfig.getSettings().getSubtitleSettings();

        invokeListeners(e -> e.onSubtitleFamilyChanged(subtitleSettings.getFontFamily().getFamily()));
        invokeListeners(e -> e.onSubtitleFontWeightChanged(subtitleSettings.isBold()));
        invokeListeners(e -> e.onSubtitleSizeChanged(subtitleSettings.getFontSize()));
        invokeListeners(e -> e.onSubtitleDecorationChanged(subtitleSettings.getDecoration()));
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    void init() {
        initializeListeners();
    }

    private void initializeListeners() {
        player.addListener(playerListener);
        applicationConfig.register(this::onSubtitleSettingsChanged);
        videoService.videoPlayerProperty().addListener((observableValue, videoPlayer, newVideoPlayer) -> onVideoViewChanged(newVideoPlayer));
        subtitleManagerService.subtitleSizeProperty().addListener((observableValue, number, newSize) -> onSubtitleSizeChanged(newSize));
        subtitleManagerService.registerListener(new SubtitleListener() {
            @Override
            public void onSubtitleChanged(Subtitle newSubtitle) {
                onActiveSubtitleChanged(newSubtitle);
            }

            @Override
            public void onSubtitleDisabled() {
                invokeListeners(PopcornPlayerSectionListener::onSubtitleDisabled);
            }
        });
    }

    //endregion

    private void onActiveSubtitleChanged(Subtitle newSubtitle) {
        if (newSubtitle == null)
            return;

        invokeListeners(e -> e.onSubtitleChanged(newSubtitle));
    }

    private void onSubtitleSizeChanged(Number newSize) {
        invokeListeners(e -> e.onSubtitleSizeChanged(newSize.intValue()));
    }

    private void onSubtitleSettingsChanged(ApplicationConfigEvent.ByValue event) {
        if (event.getTag() == ApplicationConfigEvent.Tag.SubtitleSettingsChanged) {
            var settings = event.getUnion().getSubtitleSettings().getSettings();
            invokeListeners(e -> e.onSubtitleFamilyChanged(settings.getFontFamily().getFamily()));
            invokeListeners(e -> e.onSubtitleSizeChanged(settings.getFontSize()));
            invokeListeners(e -> e.onSubtitleFontWeightChanged(settings.isBold()));
            invokeListeners(e -> e.onSubtitleDecorationChanged(settings.getDecoration()));
        }
    }

    private void onPlayerTimeChanged(long newTime) {
        invokeListeners(e -> e.onPlayerTimeChanged(newTime));
    }

    private void onPlayerStateChanged(PlayerState newState) {
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
            public void onStateChanged(PlayerState newState) {
                onPlayerStateChanged(newState);
            }

            @Override
            public void onVolumeChanged(int volume) {
                onPlayerVolumeChanged(volume);
            }
        };
    }
}
