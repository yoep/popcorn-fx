package com.github.yoep.popcorn.subtitle.controls;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.yoep.popcorn.subtitle.models.SubtitleInfo;
import javafx.application.Platform;
import javafx.collections.ObservableList;
import javafx.geometry.Bounds;
import javafx.scene.Cursor;
import javafx.scene.Node;
import javafx.scene.control.ListView;
import javafx.scene.control.PopupControl;
import javafx.scene.control.Skin;
import lombok.extern.slf4j.Slf4j;
import org.springframework.util.Assert;

import java.util.ArrayList;
import java.util.List;

/**
 * Control for selecting the language through a list of text as a popup above the control.
 * The list popup is shown when the control is clicked.
 */
@Slf4j
public class LanguageSelection extends Icon {
    private final ListPopup popup = new ListPopup();
    private final List<LanguageSelectionListener> listeners = new ArrayList<>();

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
    public ObservableList<SubtitleInfo> getItems() {
        return getListView().getItems();
    }

    /**
     * Get the {@link ListView} that is being shown in the popup.
     *
     * @return Returns the list view from the popup.
     */
    public ListView<SubtitleInfo> getListView() {
        return popup.getListView();
    }

    public void addListener(LanguageSelectionListener listener) {
        Assert.notNull(listener, "listener cannot be null");
        synchronized (listeners) {
            listeners.add(listener);
        }
    }

    public void removeListener(LanguageSelectionListener listener) {
        Assert.notNull(listener, "listener cannot be null");
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
    public void select(SubtitleInfo subtitle) {
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

    private void selectItem(SubtitleInfo newValue) {
        synchronized (listeners) {
            listeners.forEach(e -> e.onItemChanged(newValue));
        }
    }

    private class ListPopup extends PopupControl {
        private final ListView<SubtitleInfo> listView = new ListView<>();

        public ListView<SubtitleInfo> getListView() {
            return listView;
        }

        @Override
        protected Skin<?> createDefaultSkin() {
            return new ListPopupSkin(this, listView);
        }
    }

    private class ListPopupSkin implements Skin<ListPopup> {
        private final ListPopup popup;
        private final ListView<SubtitleInfo> listView;

        private ListPopupSkin(ListPopup popup, ListView<SubtitleInfo> listView) {
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
