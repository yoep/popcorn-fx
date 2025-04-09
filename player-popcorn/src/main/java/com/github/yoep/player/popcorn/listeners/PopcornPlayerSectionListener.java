package com.github.yoep.player.popcorn.listeners;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationSettings;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Player;
import com.github.yoep.popcorn.backend.subtitles.Subtitle;
import javafx.scene.Node;

public interface PopcornPlayerSectionListener {
    void onSubtitleChanged(Subtitle subtitle);

    void onSubtitleDisabled();

    void onPlayerStateChanged(Player.State state);

    void onPlayerTimeChanged(long time);

    void onSubtitleSizeChanged(int newFontSize);

    void onSubtitleFamilyChanged(String newFontFamily);

    void onSubtitleFontWeightChanged(Boolean bold);

    void onSubtitleDecorationChanged(ApplicationSettings.SubtitleSettings.DecorationType newDecorationType);

    void onVideoViewChanged(Node videoView);

    void onVolumeChanged(int volume);
}
