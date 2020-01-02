package com.github.yoep.popcorn.torrent.controls;

import javafx.application.Platform;
import javafx.scene.control.Label;

public class StreamInfoCell extends Label {
    private static final String STYLE_CLASS = "cell";

    private final String name;

    /**
     * Initialize a new instance of {@link StreamInfoCell}.
     *
     * @param name The name of the info cell.
     */
    public StreamInfoCell(String name) {
        this.name = name;

        update();
        getStyleClass().add(STYLE_CLASS);
    }

    /**
     * Update this cell instance.
     */
    private void update() {
        Platform.runLater(() -> setText(name));
    }
}
