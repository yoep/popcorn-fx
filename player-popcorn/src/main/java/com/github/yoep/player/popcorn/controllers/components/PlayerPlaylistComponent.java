package com.github.yoep.player.popcorn.controllers.components;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Playlist;
import com.github.yoep.popcorn.backend.playlists.DefaultPlaylistManager;
import com.github.yoep.popcorn.backend.playlists.PlayNext;
import com.github.yoep.popcorn.backend.playlists.PlaylistManagerListener;
import com.github.yoep.popcorn.ui.view.ViewLoader;
import com.github.yoep.popcorn.ui.view.controllers.common.components.PlaylistItemComponent;
import com.github.yoep.popcorn.ui.view.controls.PlaylistControl;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.ArrayList;
import java.util.List;
import java.util.Objects;
import java.util.ResourceBundle;

import static java.util.Arrays.asList;

@Slf4j
@RequiredArgsConstructor
public class PlayerPlaylistComponent implements Initializable {
    public static final String PLAYLIST_ITEM_COMPONENT = "common/components/playlist-item.component.fxml";

    private final DefaultPlaylistManager playlistManager;
    private final ViewLoader viewLoader;
    private final ImageService imageService;

    private Playlist.Item currentItem;

    @FXML
    PlaylistControl playlistControl;

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        playlistControl.setItemFactory(item -> {
            var controller = new PlaylistItemComponent(item, imageService);
            return viewLoader.load(PLAYLIST_ITEM_COMPONENT, controller);
        });
        playlistManager.addListener(new PlaylistManagerListener() {
            @Override
            public void onPlaylistChanged() {
                PlayerPlaylistComponent.this.onPlaylistChanged();
            }

            @Override
            public void onPlayNextChanged(PlayNext next) {
                if (Objects.requireNonNull(next) instanceof PlayNext.Next(var item)) {
                    PlayerPlaylistComponent.this.onPlayNextChanged(item);
                }
            }

            @Override
            public void onPlayNextIn(int seconds) {
                // no-op
            }

            @Override
            public void onPlayNextInAborted() {
                // no-op
            }

            @Override
            public void onStateChanged(Playlist.State state) {
                switch (state) {
                    case IDLE, STOPPED -> onPlayListItemsChanged(new ArrayList<>());
                }
            }
        });
    }

    private void onPlaylistChanged() {
        playlistManager.playlist().whenComplete((playlist, throwable) -> {
            if (throwable == null) {
                onPlayListItemsChanged(asList(playlist.getItemsList()
                        .toArray(new Playlist.Item[0])));
            } else {
                log.error("Failed to retrieve playlist", throwable);
            }
        });
    }

    private void onPlayNextChanged(Playlist.Item item) {
        onCurrentItemChanged(item);
        this.currentItem = item;
    }

    private void onCurrentItemChanged(Playlist.Item item) {
        var oldItem = this.currentItem;
        Platform.runLater(() -> {
            if (oldItem != null) {
                playlistControl.setItems(playlistControl.getItems()
                        .stream()
                        .filter(e -> e != oldItem)
                        .toList());
            }

            playlistControl.setSelectedItem(item);
        });
    }

    private void onPlayListItemsChanged(List<Playlist.Item> items) {
        Platform.runLater(() -> playlistControl.setItems(items));
        if (items.isEmpty()) {
            return;
        }

        this.currentItem = items.getFirst();
    }
}
