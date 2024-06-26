package com.github.yoep.player.popcorn.listeners;

import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import com.github.yoep.popcorn.backend.settings.models.subtitles.DecorationType;
import com.github.yoep.popcorn.backend.subtitles.Subtitle;
import javafx.scene.Node;

public interface PopcornPlayerSectionListener {
    void onSubtitleChanged(Subtitle subtitle);

    void onSubtitleDisabled();

    void onPlayerStateChanged(PlayerState state);

    void onPlayerTimeChanged(long time);

    void onSubtitleSizeChanged(int newFontSize);

    void onSubtitleFamilyChanged(String newFontFamily);

    void onSubtitleFontWeightChanged(Boolean bold);

    void onSubtitleDecorationChanged(DecorationType newDecorationType);

    void onVideoViewChanged(Node videoView);

    void onVolumeChanged(int volume);
}
