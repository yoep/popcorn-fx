package com.github.yoep.player.popcorn.controllers.components;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Playlist;
import com.github.yoep.popcorn.backend.playlists.DefaultPlaylistManager;
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
import java.util.ResourceBundle;

@Slf4j
@RequiredArgsConstructor
public class PlayerPlaylistComponent implements Initializable {
    public static final String PLAYLIST_ITEM_COMPONENT = "common/components/playlist-item.component.fxml";

    private final DefaultPlaylistManager playlistManager;
    private final ViewLoader viewLoader;
    private final ImageService imageService;

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
            public void onPlayingIn(Long playingIn, Playlist.Item item) {

            }

            @Override
            public void onStateChanged(Playlist.State state) {

            }
        });
    }

    private void onPlaylistChanged() {
        playlistManager.playlist().whenComplete((playlist, throwable) -> {
            if (throwable == null) {
                Platform.runLater(() -> playlistControl.setItems(playlist.getItemsList()));
            } else {
                log.error("Failed to retrieve playlist", throwable);
            }
        });
    }
}
