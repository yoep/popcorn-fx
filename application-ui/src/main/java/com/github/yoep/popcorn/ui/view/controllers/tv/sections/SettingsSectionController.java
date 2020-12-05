package com.github.yoep.popcorn.ui.view.controllers.tv.sections;

import com.github.yoep.popcorn.ui.view.controls.Overlay;
import com.github.yoep.popcorn.ui.view.controls.OverlayListener;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.Node;
import lombok.NoArgsConstructor;
import org.springframework.util.Assert;

import java.net.URL;
import java.util.ArrayList;
import java.util.List;
import java.util.ResourceBundle;

@NoArgsConstructor
public class SettingsSectionController implements Initializable {
    private List<OverlayListener> listenersBuffer;

    @FXML
    private Overlay overlay;

    SettingsSectionController(Overlay overlay) {
        this.overlay = overlay;
    }

    void setOverlay(Overlay overlay) {
        this.overlay = overlay;
    }

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        unloadBuffer();
    }

    /**
     * @see Overlay#setBackspaceActionEnabled(boolean)
     */
    public void setBackspaceActionEnabled(boolean backspaceActionEnabled) {
        overlay.setBackspaceActionEnabled(backspaceActionEnabled);
    }

    /**
     * @see Overlay#addListener(OverlayListener)
     */
    public void addListener(OverlayListener listener) {
        Assert.notNull(listener, "listener cannot be null");

        // check if the overlay is initialized
        // if not, add it to the cache
        if (overlay == null) {
            addToCache(listener);
        } else {
            overlay.addListener(listener);
        }
    }

    /**
     * Show the overlay on top of the settings.
     *
     * @see Overlay#show(Node, Node)
     */
    public void showOverlay(Node originNode, Node contents) {
        overlay.show(originNode, contents);
    }

    private void addToCache(OverlayListener listener) {
        if (listenersBuffer == null) {
            listenersBuffer = new ArrayList<>();
        }

        listenersBuffer.add(listener);
    }

    private void unloadBuffer() {
        if (listenersBuffer == null)
            return;

        listenersBuffer.forEach(overlay::addListener);
    }
}
