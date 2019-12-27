package com.github.yoep.popcorn.subtitle.controls;

import com.github.yoep.popcorn.subtitle.models.Subtitle;

public interface LanguageSelectionListener {
    /**
     * Invoked when the selected item is being changed.
     *
     * @param newValue The new selected item.
     */
    void onItemChanged(Subtitle newValue);
}
