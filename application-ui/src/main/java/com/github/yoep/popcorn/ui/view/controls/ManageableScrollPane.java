package com.github.yoep.popcorn.ui.view.controls;

import javafx.beans.property.BooleanProperty;
import javafx.beans.property.SimpleBooleanProperty;
import javafx.event.EventDispatcher;
import javafx.scene.Node;
import javafx.scene.control.ScrollPane;

/**
 * The {@link ManageableScrollPane} extends upon the FX {@link ScrollPane} control with additional management
 * options which are by default not exposed.
 */
public class ManageableScrollPane extends ScrollPane {
    public static final String SHORT_KEYS_PROPERTY = "shortKeysEnabled";

    /**
     * Specifies if short keys are enabled for this {@link InfiniteScrollPane}.
     * This includes the scrolling behavior with arrow keys, {@link javafx.scene.input.KeyCode#HOME}, {@link javafx.scene.input.KeyCode#END},
     * {@link javafx.scene.input.KeyCode#PAGE_UP}, etc.
     */
    private final BooleanProperty shortKeysEnabled = new SimpleBooleanProperty(this, SHORT_KEYS_PROPERTY, true);

    private EventDispatcher eventDispatcher;

    //region Constructors

    public ManageableScrollPane() {
        super();
        init();
    }

    public ManageableScrollPane(Node content) {
        super(content);
        init();
    }

    //endregion

    //region Properties

    /**
     * Check if the short keys are enabled for this {@link InfiniteScrollPane}.
     *
     * @return Returns true if the short keys are enabled, else false.
     */
    public boolean getShortKeysEnabled() {
        return shortKeysEnabled.get();
    }

    /**
     * Get the short keys property.
     *
     * @return Returns the property for the short keys.
     */
    public BooleanProperty shortKeysEnabledProperty() {
        return shortKeysEnabled;
    }

    /**
     * Specify if the short keys are enabled.
     *
     * @param shortKeysEnabled Enables the short keys.
     */
    public void setShortKeysEnabled(boolean shortKeysEnabled) {
        this.shortKeysEnabled.set(shortKeysEnabled);
    }

    //endregion

    //region Functions

    private void init() {
        initializeShortKeys();
    }

    private void initializeShortKeys() {
        shortKeysEnabledProperty().addListener((observable, oldValue, newValue) -> onShortKeysChanged(newValue));
        onShortKeysChanged(getShortKeysEnabled());
    }

    private void onShortKeysChanged(boolean enabled) {
        if (!enabled) {
            // store the existing event dispatcher for later use
            if (eventDispatcher == null)
                eventDispatcher = this.getEventDispatcher();

            this.setEventDispatcher(null);
        } else if (eventDispatcher != null) {
            this.setEventDispatcher(eventDispatcher);
        }
    }

    //endregion
}
