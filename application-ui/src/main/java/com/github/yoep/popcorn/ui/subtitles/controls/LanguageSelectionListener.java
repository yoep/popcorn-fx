package com.github.yoep.popcorn.ui.subtitles.controls;

import com.github.yoep.popcorn.ui.subtitles.models.SubtitleInfo;

public interface LanguageSelectionListener {
    /**
     * Invoked when the selected item is being changed.
     *
     * @param newValue The new selected item.
     */
    void onItemChanged(SubtitleInfo newValue);
}
