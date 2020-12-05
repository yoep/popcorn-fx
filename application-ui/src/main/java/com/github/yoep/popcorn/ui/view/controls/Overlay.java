package com.github.yoep.popcorn.ui.view.controls;

import javafx.application.Platform;
import javafx.beans.property.BooleanProperty;
import javafx.beans.property.SimpleBooleanProperty;
import javafx.event.EventHandler;
import javafx.scene.Node;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.layout.StackPane;
import lombok.extern.slf4j.Slf4j;
import org.springframework.util.Assert;

import javax.validation.constraints.NotNull;
import java.util.ArrayList;
import java.util.List;

@Slf4j
public class Overlay extends StackPane {
    public static final String STYLE_CLASS = "overlay";
    public static final String BACKSPACE_ENABLED_PROPERTY = "backspaceActionEnabled";

    private final BooleanProperty backspaceActionEnabled = new SimpleBooleanProperty(this, BACKSPACE_ENABLED_PROPERTY, true);
    private final EventHandler<KeyEvent> contentEventHandler = this::handleContentEvent;
    private final List<OverlayListener> listeners = new ArrayList<>();

    private Node originNode;
    private Node contents;

    //region Constructors

    public Overlay() {
        super();
        init();
    }

    public Overlay(Node... children) {
        super(children);
        init();
    }

    //endregion

    //region Properties

    /**
     * Verify if the backspace action key is enabled for the overlay.
     *
     * @return Returns true if the backspace action key is enabled, else false.
     */
    public boolean isBackspaceActionEnabled() {
        return backspaceActionEnabled.get();
    }

    /**
     * Get the backspace action enabled property from the overlay.
     *
     * @return Returns the backspace action property.
     */
    public BooleanProperty backspaceActionEnabledProperty() {
        return backspaceActionEnabled;
    }

    /**
     * Set if the backspace action key should be enabled.
     *
     * @param backspaceActionEnabled The value to indicate if the backspace action should be enabled.
     */
    public void setBackspaceActionEnabled(boolean backspaceActionEnabled) {
        this.backspaceActionEnabled.set(backspaceActionEnabled);
    }


    //endregion

    //region Methods

    /**
     * Register the given listener in the overlay.
     *
     * @param listener The listener to register.
     */
    public void addListener(@NotNull OverlayListener listener) {
        Assert.notNull(listener, "listener cannot be null");
        synchronized (listeners) {
            listeners.add(listener);
        }
    }

    /**
     * Unregister the given listener from the overlay.
     *
     * @param listener The listener to remove.
     */
    private void removeListener(OverlayListener listener) {
        synchronized (listeners) {
            listeners.remove(listener);
        }
    }

    /**
     * Show the overlay with the given contents.
     *
     * @param originNode The origin node which triggered this overlay.
     * @param contents   The contents to display in the overlay.
     */
    public void show(Node originNode, Node contents) {
        Assert.notNull(originNode, "originNode cannot be null");
        Assert.notNull(contents, "contents cannot be null");
        this.originNode = originNode;
        this.contents = contents;

        contents.addEventHandler(KeyEvent.ANY, contentEventHandler);
        getChildren().clear();
        getChildren().add(contents);

        setVisible(true);
        Platform.runLater(contents::requestFocus);
    }

    //endregion

    //region Functions

    private void init() {
        log.trace("Overlay control is being initialized");
        initializeStyle();
        initializeEvents();

        setVisible(false);
    }

    private void initializeStyle() {
        getStyleClass().add(STYLE_CLASS);
    }

    private void initializeEvents() {
        this.setOnKeyPressed(this::onKeyEvent);
    }

    private void handleContentEvent(KeyEvent event) {
        if (event.getEventType() != KeyEvent.KEY_RELEASED)
            onKeyEvent(event);
    }

    private void onKeyEvent(KeyEvent event) {
        var code = event.getCode();

        if (shouldCloseOverlay(code)) {
            event.consume();
            onClose();
        }
    }

    private boolean shouldCloseOverlay(KeyCode code) {
        return code == KeyCode.ENTER || code == KeyCode.ESCAPE ||
                (isBackspaceActionEnabled() && code == KeyCode.BACK_SPACE);
    }

    private void onClose() {
        log.trace("Overlay control is being closed");
        setVisible(false);

        contents.removeEventHandler(KeyEvent.ANY, contentEventHandler);
        originNode.requestFocus();

        synchronized (listeners) {
            listeners.forEach(OverlayListener::onClose);
        }
    }

    //endregion
}
