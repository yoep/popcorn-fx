package com.github.yoep.popcorn.controls;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.yoep.popcorn.media.providers.models.Episode;

public interface EpisodesListener {
    /**
     * Invoked when the icon is clicked.
     *
     * @param icon    The icon that has been clicked.
     * @param episode The episode behind the icon that is clicked.
     */
    void onIconClicked(Icon icon, Episode episode);
}
