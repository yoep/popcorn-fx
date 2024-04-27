package com.github.yoep.player.popcorn.controllers.components;

import com.github.yoep.popcorn.backend.playlists.PlaylistItem;
import com.github.yoep.popcorn.backend.playlists.PlaylistManager;
import com.github.yoep.popcorn.backend.playlists.PlaylistManagerListener;
import com.github.yoep.popcorn.backend.playlists.PlaylistState;
import com.github.yoep.popcorn.ui.view.ViewLoader;
import com.github.yoep.popcorn.ui.view.controllers.common.components.PlaylistItemComponent;
import com.github.yoep.popcorn.ui.view.controls.PlaylistControl;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
@RequiredArgsConstructor
public class PlayerPlaylistComponent implements Initializable {
    private final PlaylistManager playlistManager;
    private final ViewLoader viewLoader;
    private final ImageService imageService;

    @FXML
    PlaylistControl playlistControl;

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        playlistControl.setItemFactory(item -> {
            var controller = new PlaylistItemComponent(item, imageService);
            return viewLoader.load("common/components/playlist-item.component.fxml", controller);
        });
        playlistManager.addListener(new PlaylistManagerListener() {
            @Override
            public void onPlaylistChanged() {
                PlayerPlaylistComponent.this.onPlaylistChanged();
            }

            @Override
            public void onPlayingIn(Long playingIn, PlaylistItem item) {

            }

            @Override
            public void onStateChanged(PlaylistState state) {

            }
        });
    }

    private void onPlaylistChanged() {
        try (var playlist = playlistManager.playlist()) {
            playlistControl.setItems(playlist.getItems());
        }
    }
}
