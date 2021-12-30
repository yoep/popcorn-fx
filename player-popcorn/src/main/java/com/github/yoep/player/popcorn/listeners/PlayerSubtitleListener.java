package com.github.yoep.player.popcorn.listeners;

import com.github.yoep.popcorn.backend.subtitles.models.SubtitleInfo;

import java.util.List;

public interface PlayerSubtitleListener {
    void onActiveSubtitleChanged(SubtitleInfo activeSubtitle);

    void onAvailableSubtitlesChanged(List<SubtitleInfo> subtitles, SubtitleInfo activeSubtitle);
}
