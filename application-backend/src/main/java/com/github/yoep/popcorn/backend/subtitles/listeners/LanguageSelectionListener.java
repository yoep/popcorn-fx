package com.github.yoep.popcorn.backend.subtitles.listeners;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Subtitle;

public interface LanguageSelectionListener {
    /**
     * Invoked when the selected item is being changed.
     *
     * @param newValue The new selected item.
     */
    void onItemChanged(Subtitle.Info newValue);
}
