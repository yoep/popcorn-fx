package com.github.yoep.popcorn.ui.view.controls;

import com.github.spring.boot.javafx.font.controls.Icon;
import javafx.application.Platform;
import javafx.beans.property.StringProperty;
import javafx.geometry.Insets;
import javafx.geometry.Pos;
import javafx.scene.Cursor;
import javafx.scene.control.TextField;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.StackPane;
import lombok.extern.slf4j.Slf4j;
import org.springframework.util.Assert;

import java.util.ArrayList;
import java.util.List;

@Slf4j
public class SearchField extends StackPane {
    private static final int MILLIS_BETWEEN_INVOKES = 500;
    private static final int WATCHER_TTL = 5000;
    private static final String STYLE_CLASS = "search-field";
    private static final String STYLE_CLASS_SEARCH_ICON = "search-icon";
    private static final String STYLE_CLASS_CLOSE_ICON = "clear-icon";
    private static final String STYLE_CLASS_TEXT_FIELD = "textfield";

    private final List<SearchListener> listeners = new ArrayList<>();

    private Icon searchIcon;
    private Icon clearIcon;
    private TextField textField;

    private boolean keepWatcherAlive;
    private long lastChangeInvoked;
    private long lastUserInput;

    public SearchField() {
        init();
    }

    /**
     * Get the text prompt property of this search field.
     *
     * @return Returns the text prompt property.
     */
    public StringProperty promptTextProperty() {
        return this.textField.promptTextProperty();
    }

    /**
     * Get the current prompt text.
     *
     * @return Returns the current prompt text.
     */
    public String getPromptText() {
        return this.textField.getPromptText();
    }

    /**
     * Set the prompt text of this search field.
     *
     * @param value The prompt text to set.
     */
    public void setPromptText(String value) {
        this.textField.setPromptText(value);
    }

    /**
     * Get the text property of this search field.
     *
     * @return Returns the text property.
     */
    public StringProperty textProperty() {
        return this.textField.textProperty();
    }

    public String getText() {
        return this.textField.getText();
    }

    public void setText(String text) {
        this.textField.setText(text);
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
        Platform.runLater(() -> textField.setText(""));
        onCleared();
    }

    private void init() {
        initializeTextField();
        initializeSearchIcon();
        initializeCloseIcon();
        initializeStyles();
        initializeListener();
    }

    private void initializeTextField() {
        textField = new TextField();
        textField.setPadding(new Insets(2, 25, 2, 25));
        getChildren().add(textField);
    }

    private void initializeSearchIcon() {
        searchIcon = new Icon(Icon.SEARCH_UNICODE);
        searchIcon.setCursor(Cursor.HAND);
        StackPane.setAlignment(searchIcon, Pos.CENTER_LEFT);
        getChildren().add(searchIcon);
    }

    private void initializeCloseIcon() {
        clearIcon = new Icon(Icon.TIMES_UNICODE);
        clearIcon.setCursor(Cursor.HAND);
        clearIcon.setOnMouseClicked(this::onClearClicked);
        clearIcon.setVisible(false);
        StackPane.setAlignment(clearIcon, Pos.CENTER_RIGHT);
        getChildren().add(clearIcon);
    }

    private void initializeStyles() {
        this.getStyleClass().add(STYLE_CLASS);
        this.searchIcon.getStyleClass().add(STYLE_CLASS_SEARCH_ICON);
        this.clearIcon.getStyleClass().add(STYLE_CLASS_CLOSE_ICON);
        this.textField.getStyleClass().add(STYLE_CLASS_TEXT_FIELD);
    }

    private void initializeListener() {
        textProperty().addListener((observable, oldValue, newValue) -> {
            clearIcon.setVisible(newValue.length() > 0);
            lastUserInput = System.currentTimeMillis();

            if (!keepWatcherAlive)
                createWatcher();
        });
    }

    private void onChanged() {
        String value = textField.getText();
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

                    Thread.sleep(100);
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
}
