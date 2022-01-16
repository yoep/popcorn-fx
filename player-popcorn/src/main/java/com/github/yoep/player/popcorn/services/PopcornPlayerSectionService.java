package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.popcorn.listeners.AbstractPlayerListener;
import com.github.yoep.player.popcorn.listeners.PopcornPlayerSectionListener;
import com.github.yoep.player.popcorn.player.PopcornPlayer;
import com.github.yoep.popcorn.backend.adapters.player.listeners.PlayerListener;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import com.github.yoep.popcorn.backend.adapters.screen.ScreenService;
import com.github.yoep.popcorn.backend.adapters.video.VideoPlayer;
import com.github.yoep.popcorn.backend.services.AbstractListenerService;
import com.github.yoep.popcorn.backend.settings.SettingsService;
import com.github.yoep.popcorn.backend.settings.models.SubtitleSettings;
import com.github.yoep.popcorn.backend.settings.models.subtitles.DecorationType;
import com.github.yoep.popcorn.backend.subtitles.Subtitle;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

import javax.annotation.PostConstruct;
import java.beans.PropertyChangeEvent;

@Slf4j
@Service
@RequiredArgsConstructor
public class PopcornPlayerSectionService extends AbstractListenerService<PopcornPlayerSectionListener> {
    private final PopcornPlayer player;
    private final ScreenService screenService;
    private final SettingsService settingsService;
    private final SubtitleManagerService subtitleManagerService;
    private final VideoService videoService;

    private final PlayerListener playerListener = createPlayerListener();

    //region Methods

    public boolean isNativeSubtitlePlaybackSupported() {
        return videoService.getVideoPlayer()
                .map(VideoPlayer::supportsNativeSubtitleFile)
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

    public void provideSubtitleValues() {
        var subtitleSettings = settingsService.getSettings().getSubtitleSettings();

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
        settingsService.getSettings().getSubtitleSettings().addListener(this::onSubtitleSettingsChanged);
        videoService.videoPlayerProperty().addListener((observableValue, videoPlayer, newVideoPlayer) -> onVideoViewChanged(newVideoPlayer));
        subtitleManagerService.subtitleSizeProperty().addListener((observableValue, number, newSize) -> onSubtitleSizeChanged(newSize));
        subtitleManagerService.activeSubtitleProperty().addListener((observableValue, subtitle, newSubtitle) -> onActiveSubtitleChanged(newSubtitle));
    }

    //endregion

    private void onActiveSubtitleChanged(Subtitle newSubtitle) {
        invokeListeners(e -> e.onSubtitleChanged(newSubtitle));
    }

    private void onSubtitleSizeChanged(Number newSize) {
        invokeListeners(e -> e.onSubtitleSizeChanged(newSize.intValue()));
    }

    private void onSubtitleSettingsChanged(PropertyChangeEvent evt) {
        switch (evt.getPropertyName()) {
            case SubtitleSettings.FONT_FAMILY_PROPERTY:
                invokeListeners(e -> e.onSubtitleFamilyChanged((String) evt.getNewValue()));
                break;
            case SubtitleSettings.FONT_SIZE_PROPERTY:
                invokeListeners(e -> e.onSubtitleSizeChanged((Integer) evt.getNewValue()));
                break;
            case SubtitleSettings.BOLD_PROPERTY:
                var bold = (Boolean) evt.getNewValue();
                invokeListeners(e -> e.onSubtitleFontWeightChanged(bold));
                break;
            case SubtitleSettings.DECORATION_PROPERTY:
                invokeListeners(e -> e.onSubtitleDecorationChanged((DecorationType) evt.getNewValue()));
                break;
        }
    }

    private void onPlayerTimeChanged(long newTime) {
        invokeListeners(e -> e.onPlayerTimeChanged(newTime));
    }

    private void onPlayerStateChanged(PlayerState newState) {
        log.trace("Video player entered state {}", newState);
        invokeListeners(e -> e.onPlayerStateChanged(newState));
    }

    private void onVideoViewChanged(VideoPlayer newVideoPlayer) {
        invokeListeners(e -> e.onVideoViewChanged(newVideoPlayer.getVideoSurface()));
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
        };
    }
}
