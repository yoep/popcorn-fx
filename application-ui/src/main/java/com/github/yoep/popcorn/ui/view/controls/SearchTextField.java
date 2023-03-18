package com.github.yoep.popcorn.ui.view.controls;

import com.github.spring.boot.javafx.font.controls.Icon;
import javafx.geometry.Pos;
import javafx.scene.Cursor;
import javafx.scene.control.Control;
import javafx.scene.control.Skin;
import javafx.scene.control.TextField;
import javafx.scene.control.skin.TextFieldSkin;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.Pane;
import lombok.extern.slf4j.Slf4j;
import org.springframework.util.Assert;

import java.util.ArrayList;
import java.util.List;

@Slf4j
public class SearchTextField extends TextField {
    private static final int MILLIS_BETWEEN_INVOKES = 500;
    private static final int WATCHER_TTL = 5000;
    private static final String STYLE_CLASS = "search-field";
    private static final String INNER_STYLE_CLASS = "value";
    private static final String STYLE_CLASS_CLOSE_ICON = "clear-icon";

    private final List<SearchListener> listeners = new ArrayList<>();

    private boolean keepWatcherAlive;
    private long lastChangeInvoked;
    private long lastUserInput;

    public SearchTextField() {
        super();
        skinProperty().addListener((observable, oldValue, newValue) -> init());
    }

    /**
     * Register the given listener to this instance.
     *
     * @param listener The listener to register.
     */
    public void addListener(SearchListener listener) {
        Assert.notNull(listener, "listener cannot be null");
        synchronized (listeners) {
            listeners.add(listener);
        }
    }

    /**
     * Remove the given listener from this instance.
     *
     * @param listener The listener to remove.
     */
    public void removeListener(SearchListener listener) {
        Assert.notNull(listener, "listener cannot be null");
        synchronized (listeners) {
            listeners.remove(listener);
        }
    }

    /**
     * Clear the current search value.
     * This will reset the search to nothing.
     */
    public void clear() {
        setText("");
        onCleared();
    }

    @Override
    protected Skin<?> createDefaultSkin() {
        return new SearchTextFieldSkin(this);
    }

    private void init() {
        initializeTextField();
        initializeStyles();
        initializeListener();
    }

    private void initializeTextField() {
        var innerPane = (Pane) (getChildren().get(0));
        innerPane.getStyleClass().add(INNER_STYLE_CLASS);
    }

    private void initializeStyles() {
        this.getStyleClass().add(STYLE_CLASS);
    }

    private void initializeListener() {
        getSearchSkin().clearIcon.setOnMouseClicked(this::onClearClicked);
        textProperty().addListener((observable, oldValue, newValue) -> {
            getSearchSkin().clearIcon.setVisible(newValue.length() > 0);
            lastUserInput = System.currentTimeMillis();

            if (!keepWatcherAlive)
                createWatcher();
        });
    }

    private SearchTextFieldSkin getSearchSkin() {
        return (SearchTextFieldSkin) getSkin();
    }

    private void onChanged() {
        var value = getText();
        lastChangeInvoked = System.currentTimeMillis();

        synchronized (listeners) {
            for (SearchListener listener : listeners) {
                try {
                    listener.onSearchValueChanged(value);
                } catch (Exception ex) {
                    log.error(ex.getMessage(), ex);
                }
            }
        }
    }

    private void onCleared() {
        lastChangeInvoked = System.currentTimeMillis();

        synchronized (listeners) {
            for (SearchListener listener : listeners) {
                try {
                    listener.onSearchValueCleared();
                } catch (Exception ex) {
                    log.error(ex.getMessage(), ex);
                }
            }
        }
    }

    private void onClearClicked(MouseEvent event) {
        event.consume();
        clear();
    }

    private void createWatcher() {
        keepWatcherAlive = true;

        runTask(() -> {
            try {
                while (keepWatcherAlive) {
                    if (isOnChangeInvocationAllowed()) {
                        onChanged();
                    } else if (isOnClearInvocationAllowed()) {
                        onCleared();
                    }

                    // stop the watcher if the last user interaction was more than #WATCHER_TTL millis ago
                    if (System.currentTimeMillis() - lastUserInput > WATCHER_TTL)
                        keepWatcherAlive = false;

                    Thread.sleep(50);
                }
            } catch (InterruptedException ex) {
                log.error(ex.getMessage(), ex);
            }
        });
    }

    private boolean isOnChangeInvocationAllowed() {
        int noSpacesText = getText().trim().length();

        return noSpacesText >= 3 &&
                isInvocationAllowedBasedOnTime() &&
                lastUserInput > lastChangeInvoked;
    }

    private boolean isOnClearInvocationAllowed() {
        int noSpacesText = getText().trim().length();

        return noSpacesText < 3 &&
                isInvocationAllowedBasedOnTime() &&
                lastUserInput > lastChangeInvoked;
    }

    private boolean isInvocationAllowedBasedOnTime() {
        long currentTimeMillis = System.currentTimeMillis();

        return currentTimeMillis - lastUserInput > 300 &&
                currentTimeMillis - lastChangeInvoked > MILLIS_BETWEEN_INVOKES;
    }

    private void runTask(Runnable task) {
        new Thread(task, "SearchField").start();
    }

    private static class SearchTextFieldSkin extends TextFieldSkin {
        public Icon clearIcon;

        /**
         * Creates a new TextFieldSkin instance, installing the necessary child
         * nodes into the Control {@link Control#getChildren() children} list, as
         * well as the necessary input mappings for handling key, mouse, etc events.
         *
         * @param control The control that this skin should be installed onto.
         */
        public SearchTextFieldSkin(TextField control) {
            super(control);

            clearIcon = new Icon(Icon.TIMES_UNICODE);
            clearIcon.setCursor(Cursor.HAND);
            clearIcon.setVisible(false);
            clearIcon.setAlignment(Pos.CENTER_RIGHT);
            clearIcon.getStyleClass().add(STYLE_CLASS_CLOSE_ICON);
            ((Pane) getChildren().get(0)).getChildren().add(clearIcon);

            control.widthProperty().addListener((observable, oldValue, newValue) -> updateIconPos(newValue.doubleValue()));
            updateIconPos(control.getWidth());
        }

        private void updateIconPos(double width) {
            clearIcon.setLayoutX(width - clearIcon.getWidth());
        }
    }
}
