package com.github.yoep.player.popcorn.listeners;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Subtitle;

import java.util.List;

public interface PlayerSubtitleListener {
    void onActiveSubtitleChanged(Subtitle.Info activeSubtitle);

    void onAvailableSubtitlesChanged(List<Subtitle.Info> subtitles, Subtitle.Info activeSubtitle);
}
