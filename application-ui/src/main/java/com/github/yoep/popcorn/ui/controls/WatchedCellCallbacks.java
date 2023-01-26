package com.github.yoep.popcorn.ui.controls;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.yoep.popcorn.backend.media.providers.models.Media;

public interface WatchedCellCallbacks {
    void updateWatchedState(Media media, boolean newState, Icon icon);
}
