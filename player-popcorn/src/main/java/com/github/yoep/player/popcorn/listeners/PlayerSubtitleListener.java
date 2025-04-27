package com.github.yoep.player.popcorn.listeners;

import com.github.yoep.popcorn.backend.subtitles.ISubtitleInfo;

import java.util.List;

public interface PlayerSubtitleListener {
    void onActiveSubtitleChanged(ISubtitleInfo activeSubtitle);

    void onAvailableSubtitlesChanged(List<ISubtitleInfo> subtitles, ISubtitleInfo activeSubtitle);
}
