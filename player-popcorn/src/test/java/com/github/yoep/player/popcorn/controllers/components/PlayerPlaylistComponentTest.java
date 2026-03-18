package com.github.yoep.player.popcorn.controllers.components;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Playlist;
import com.github.yoep.popcorn.backend.playlists.DefaultPlaylistManager;
import com.github.yoep.popcorn.backend.playlists.PlaylistManagerListener;
import com.github.yoep.popcorn.ui.view.ViewLoader;
import com.github.yoep.popcorn.ui.view.controls.PlaylistControl;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.scene.layout.Pane;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;
import org.testfx.util.WaitForAsyncUtils;

import java.net.URL;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class PlayerPlaylistComponentTest {
    @Mock
    private DefaultPlaylistManager playlistManager;
    @Mock
    private ViewLoader viewLoader;
    @Mock
    private ImageService imageService;
    @Mock
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;
    private PlayerPlaylistComponent component;

    @BeforeEach
    void setUp() {
        component = new PlayerPlaylistComponent(playlistManager, viewLoader, imageService);

        component.playlistControl = new PlaylistControl();
    }

    @Test
    void testOnPlayListChanged() throws TimeoutException {
        var listenerHolder = new AtomicReference<PlaylistManagerListener>();
        var playlist = Playlist.newBuilder()
                .addItems(Playlist.Item.newBuilder()
                        .setTitle("Foo")
                        .build())
                .build();
        doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(0, PlaylistManagerListener.class));
            return null;
        }).when(playlistManager).addListener(isA(PlaylistManagerListener.class));
        when(viewLoader.load(eq(PlayerPlaylistComponent.PLAYLIST_ITEM_COMPONENT), isA(Object.class)))
                .thenReturn(new Pane());
        when(playlistManager.playlist()).thenReturn(CompletableFuture.completedFuture(playlist));
        component.initialize(url, resourceBundle);

        var listener = listenerHolder.get();
        listener.onPlaylistChanged();
        WaitForAsyncUtils.waitFor(500, TimeUnit.MILLISECONDS, () -> !component.playlistControl.getItems().isEmpty());

        verify(playlistManager).playlist();
        assertEquals(1, component.playlistControl.getItems().size());
    }
}