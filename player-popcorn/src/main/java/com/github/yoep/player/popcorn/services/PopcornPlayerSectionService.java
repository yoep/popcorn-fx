package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.popcorn.listeners.PopcornPlayerSectionListener;
import com.github.yoep.popcorn.backend.adapters.screen.ScreenService;
import com.github.yoep.popcorn.backend.settings.SettingsService;
import com.github.yoep.popcorn.backend.settings.models.SubtitleSettings;
import com.github.yoep.popcorn.backend.settings.models.subtitles.DecorationType;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

import javax.annotation.PostConstruct;
import java.beans.PropertyChangeEvent;

@Slf4j
@Service
@RequiredArgsConstructor
public class PopcornPlayerSectionService extends AbstractListenerService<PopcornPlayerSectionListener> {
    private final PlaybackService playbackService;
    private final ScreenService screenService;
    private final SettingsService settingsService;
    private final SubtitleManagerService subtitleManagerService;

    //region Methods

    public void togglePlayerPlaybackState() {
        playbackService.togglePlayerPlaybackState();
    }

    public void toggleFullscreen() {
        screenService.toggleFullscreen();
    }

    public void videoTimeOffset(int offset) {
        playbackService.videoTimeOffset(offset);
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    void init() {
        initializeListeners();
        initializeSubtitlesValues();
    }

    private void initializeListeners() {
        settingsService.getSettings().getSubtitleSettings().addListener(this::onSubtitleSettingsChanged);
        subtitleManagerService.subtitleSizeProperty().addListener((observableValue, number, newSize) -> onSubtitleSizeChanged(newSize));
    }

    private void initializeSubtitlesValues() {
        var subtitleSettings = settingsService.getSettings().getSubtitleSettings();

        invokeListeners(e -> e.onSubtitleFamilyChanged(subtitleSettings.getFontFamily().getFamily()));
        invokeListeners(e -> e.onSubtitleFontWeightChanged(subtitleSettings.isBold()));
        invokeListeners(e -> e.onSubtitleSizeChanged(subtitleSettings.getFontSize()));
    }

    //endregion

    private void onSubtitleSizeChanged(Number newSize) {
        invokeListeners(e -> e.onSubtitleSizeChanged(newSize.intValue()));
    }

    private void onSubtitleSettingsChanged(PropertyChangeEvent evt) {
        switch (evt.getPropertyName()) {
            case SubtitleSettings.FONT_FAMILY_PROPERTY:
                invokeListeners(e -> e.onSubtitleFamilyChanged((String) evt.getNewValue()));
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
}
