package com.github.yoep.popcorn.backend.subtitles;

public interface LanguageSelectionListener {
    /**
     * Invoked when the selected item is being changed.
     *
     * @param newValue The new selected item.
     */
    void onItemChanged(ISubtitleInfo newValue);
}
