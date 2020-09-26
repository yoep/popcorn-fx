package com.github.yoep.popcorn.ui.view.controllers.tv.sections;

import com.github.yoep.popcorn.ui.view.controls.Overlay;
import javafx.fxml.FXML;
import javafx.scene.Node;

public class SettingsSectionController {
    @FXML
    private Overlay overlay;

    /**
     * Show the overlay on top of the settings.
     *
     * @see Overlay#show(Node, Node)
     */
    public void showOverlay(Node originNode, Node contents) {
        overlay.show(originNode, contents);
    }
}
