package com.github.yoep.popcorn.ui.view.controllers.tv.sections;

import com.github.yoep.popcorn.ui.events.CloseSettingsEvent;
import com.github.yoep.popcorn.ui.view.controls.Overlay;
import com.github.yoep.popcorn.ui.view.controls.OverlayListener;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.Node;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import lombok.RequiredArgsConstructor;
import org.springframework.context.ApplicationEventPublisher;
import org.springframework.util.Assert;

import java.net.URL;
import java.util.ArrayList;
import java.util.List;
import java.util.ResourceBundle;

@RequiredArgsConstructor
public class SettingsSectionController implements Initializable {
    private final ApplicationEventPublisher eventPublisher;

    private List<OverlayListener> listenersBuffer;

    @FXML
    private Overlay overlay;

    SettingsSectionController(ApplicationEventPublisher eventPublisher, Overlay overlay) {
        this.eventPublisher = eventPublisher;
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

        // unload the buffer to the overlay
        // and clean the buffer as we don't need it anymore
        listenersBuffer.forEach(overlay::addListener);
        listenersBuffer = null;
    }

    private void onClose() {
        eventPublisher.publishEvent(new CloseSettingsEvent(this));
    }

    @FXML
    private void onSettingsKeyPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.BACK_SPACE || event.getCode() == KeyCode.UNDEFINED) {
            event.consume();
            onClose();
        }
    }
}
