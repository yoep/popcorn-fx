package com.github.yoep.player.popcorn.listeners;

import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import com.github.yoep.popcorn.backend.settings.models.subtitles.DecorationType;
import com.github.yoep.popcorn.backend.subtitles.Subtitle;

public interface PopcornPlayerSectionListener {
    void onSubtitleChanged(Subtitle subtitle);

    void onPlayerStateChanged(PlayerState state);

    void onPlayerTimeChanged(long time);

    void onSubtitleSizeChanged(int newFontSize);

    void onSubtitleFamilyChanged(String newFontFamily);

    void onSubtitleFontWeightChanged(Boolean bold);

    void onSubtitleDecorationChanged(DecorationType newDecorationType);
}
