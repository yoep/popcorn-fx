package com.github.yoep.popcorn.ui.view.controls;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Playlist;
import com.github.yoep.popcorn.ui.font.controls.Icon;
import javafx.collections.MapChangeListener;
import javafx.scene.Cursor;
import javafx.scene.Node;
import javafx.scene.control.PopupControl;
import javafx.scene.control.ScrollPane;
import javafx.scene.control.Skin;
import lombok.extern.slf4j.Slf4j;

import java.util.List;

@Slf4j
public class PlaylistControl extends Icon {
    static final String STYLE_CLASS = "playlist";
    static final String POPUP_STYLE_CLASS = "playlist-popup";

    private final ListPopup popup = new ListPopup();

    private boolean firstRender = true;

    public PlaylistControl() {
        initializeLanguageSelection();
    }

    public PlaylistControl(String unicode) {
        super(unicode);
        initializeLanguageSelection();
    }

    public List<Playlist.Item> getItems() {
        return getListView().getItems();
    }

    public AxisItemSelection<Playlist.Item> getListView() {
        return popup.getPlaylistView();
    }

    public void setItemFactory(ItemFactory<Playlist.Item> itemFactory) {
        popup.playlistView.setItemFactory(itemFactory);
    }

    public void show() {
        var screenBounds = this.localToScreen(this.getBoundsInLocal());
        var x = screenBounds.getMaxX();
        var y = screenBounds.getMinY();

        y -= popup.getPlaylistView().getHeight();
        x -= popup.getPlaylistView().getWidth();

        popup.show(this, x, y);

        if (firstRender) {
            firstRender = false;
            show();
        }
    }

    public void setItems(List<Playlist.Item> items) {
        popup.playlistView.setItems(items.toArray(new Playlist.Item[0]));
    }

    private void initializeLanguageSelection() {
        getStyleClass().add(STYLE_CLASS);
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
    }

    private void onClicked() {
        if (popup.isShowing()) {
            popup.hide();
        } else {
            show();
        }
    }

    private class ListPopup extends PopupControl {
        private final AxisItemSelection<Playlist.Item> playlistView = new AxisItemSelection<>();

        public ListPopup() {
            init();
        }

        public AxisItemSelection<Playlist.Item> getPlaylistView() {
            return playlistView;
        }

        @Override
        protected Skin<?> createDefaultSkin() {
            return new ListPopupSkin(this, playlistView);
        }

        private void init() {
            getStyleClass().add(POPUP_STYLE_CLASS);
            playlistView.setSpacing(1.5);
            playlistView.setVbarPolicy(ScrollPane.ScrollBarPolicy.AS_NEEDED);
            playlistView.setHbarPolicy(ScrollPane.ScrollBarPolicy.NEVER);
            playlistView.getObservableItems().addListener((MapChangeListener<? super Playlist.Item, ? super Node>) c -> {
                int height;

                if (playlistView.getItems().size() > 1) {
                    height = 275;
                } else {
                    height = 100;
                }

                playlistView.setPrefHeight(height);
                playlistView.setMaxHeight(height);
                playlistView.setMinHeight(height);
            });
        }
    }

    private class ListPopupSkin implements Skin<ListPopup> {
        private final ListPopup popup;
        private final AxisItemSelection<Playlist.Item> listView;

        private ListPopupSkin(ListPopup popup, AxisItemSelection<Playlist.Item> listView) {
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
