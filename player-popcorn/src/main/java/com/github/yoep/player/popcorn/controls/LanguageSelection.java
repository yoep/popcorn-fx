package com.github.yoep.player.popcorn.controls;

import com.github.yoep.popcorn.backend.subtitles.ISubtitleInfo;
import com.github.yoep.popcorn.backend.subtitles.listeners.LanguageSelectionListener;
import com.github.yoep.popcorn.ui.font.controls.Icon;
import javafx.application.Platform;
import javafx.collections.ListChangeListener;
import javafx.collections.ObservableList;
import javafx.geometry.Bounds;
import javafx.scene.Cursor;
import javafx.scene.Node;
import javafx.scene.control.ListView;
import javafx.scene.control.PopupControl;
import javafx.scene.control.Skin;
import lombok.extern.slf4j.Slf4j;

import java.util.ArrayList;
import java.util.List;
import java.util.Objects;

/**
 * Control for selecting the language through a list of text as a popup above the control.
 * The list popup is shown when the control is clicked.
 */
@Slf4j
public class LanguageSelection extends Icon {
    private final ListPopup popup = new ListPopup();
    private final List<LanguageSelectionListener> listeners = new ArrayList<>();

    private boolean firstRender = true;

    public LanguageSelection() {
        super();
        initializeLanguageSelection();
    }

    public LanguageSelection(String unicode) {
        super(unicode);
        initializeLanguageSelection();
    }

    /**
     * Get the items of this language selection.
     *
     * @return Returns the items of this instance.
     */
    public ObservableList<ISubtitleInfo> getItems() {
        return getListView().getItems();
    }

    /**
     * Get the {@link ListView} that is being shown in the popup.
     *
     * @return Returns the list view from the popup.
     */
    public ListView<ISubtitleInfo> getListView() {
        return popup.getListView();
    }

    /**
     * Check if the popup is currently being shown.
     *
     * @return Returns true if the popup is showing, else false.
     */
    public boolean isShowing() {
        return popup.isShowing();
    }

    public void addListener(LanguageSelectionListener listener) {
        Objects.requireNonNull(listener, "listener cannot be null");
        synchronized (listeners) {
            listeners.add(listener);
        }
    }

    public void removeListener(LanguageSelectionListener listener) {
        Objects.requireNonNull(listener, "listener cannot be null");
        synchronized (listeners) {
            listeners.remove(listener);
        }
    }

    /**
     * Select the item through the given index.
     *
     * @param index The index of the item to select.
     */
    public void select(int index) {
        getListView().getSelectionModel().select(index);
    }

    /**
     * Select the given item in the list view.
     *
     * @param subtitle The subtitle item to select.
     */
    public void select(ISubtitleInfo subtitle) {
        Platform.runLater(() -> getListView().getSelectionModel().select(subtitle));
    }

    /**
     * Show the language selection popup of this control.
     */
    public void show() {
        Bounds screenBounds = this.localToScreen(this.getBoundsInLocal());
        double x = screenBounds.getMaxX();
        double y = screenBounds.getMinY();

        y -= popup.getListView().getHeight();
        x -= popup.getListView().getWidth();

        popup.show(this, x, y);

        if (firstRender) {
            firstRender = false;
            show();
        }
    }

    private void initializeLanguageSelection() {
        initializeControl();
        initializePopup();
        initializeEvents();
    }

    private void initializeControl() {
        this.setCursor(Cursor.HAND);
    }

    private void initializePopup() {
        popup.setAutoHide(true);
        popup.setAutoFix(true);
    }

    private void initializeEvents() {
        setOnMouseClicked(event -> onClicked());
        popup.getListView().getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> {
            selectItem(newValue);
        });
    }

    private void onClicked() {
        if (popup.isShowing()) {
            popup.hide();
        } else {
            show();
        }
    }

    private void selectItem(ISubtitleInfo newValue) {
        synchronized (listeners) {
            listeners.forEach(e -> e.onItemChanged(newValue));
        }
    }

    private class ListPopup extends PopupControl {
        private final ListView<ISubtitleInfo> listView = new ListView<>();

        public ListPopup() {
            init();
        }

        public ListView<ISubtitleInfo> getListView() {
            return listView;
        }

        @Override
        protected Skin<?> createDefaultSkin() {
            return new ListPopupSkin(this, listView);
        }

        private void init() {
            listView.getItems().addListener((ListChangeListener<? super ISubtitleInfo>) c -> {
                int height;

                if (listView.getItems().size() > 1) {
                    height = 200;
                } else {
                    height = 25;
                }

                listView.setPrefHeight(height);
                listView.setMaxHeight(height);
                listView.setMinHeight(height);
            });
        }
    }

    private class ListPopupSkin implements Skin<ListPopup> {
        private final ListPopup popup;
        private final ListView<ISubtitleInfo> listView;

        private ListPopupSkin(ListPopup popup, ListView<ISubtitleInfo> listView) {
            this.popup = popup;
            this.listView = listView;
        }

        @Override
        public ListPopup getSkinnable() {
            return popup;
        }

        @Override
        public Node getNode() {
            return listView;
        }

        @Override
        public void dispose() {
        }
    }
}
