package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.playlists.PlaylistItem;
import com.github.yoep.popcorn.backend.playlists.PlaylistManager;
import com.github.yoep.popcorn.backend.playlists.PlaylistManagerListener;
import com.github.yoep.popcorn.ui.view.controls.SizedImageView;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.scene.control.Label;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.Pane;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;
import org.testfx.util.WaitForAsyncUtils;

import java.net.URL;
import java.util.Objects;
import java.util.Optional;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;
import java.util.concurrent.atomic.AtomicReference;

import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class PlayingNextInComponentTest {
    @Mock
    private ImageService imageService;
    @Mock
    private PlaylistManager playlistManager;
    @Mock
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;
    @InjectMocks
    private PlayingNextInComponent component;

    @BeforeEach
    void setUp() {
        component.playNextPane = new Pane();
        component.playNextPoster = new SizedImageView();
        component.showName = new Label();
        component.episodeTitle = new Label();
        component.episodeNumber = new Label();
        component.playingInCountdown = new Label();
    }

    @Test
    void testOnNextEpisodeChanged() throws TimeoutException {
        var title = "MyTitle";
        var caption = "MyCaption";
        var thumb = "MyThumbUrl";
        var listenerHolder = new AtomicReference<PlaylistManagerListener>();
        var item = mock(PlaylistItem.class);
        when(item.getTitle()).thenReturn(title);
        when(item.getCaption()).thenReturn(Optional.of(caption));
        when(item.getThumb()).thenReturn(Optional.of(thumb));
        when(imageService.load(isA(String.class))).thenReturn(new CompletableFuture<>());
        doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(0, PlaylistManagerListener.class));
            return null;
        }).when(playlistManager).addListener(isA(PlaylistManagerListener.class));
        component.initialize(url, resourceBundle);

        var listener = listenerHolder.get();
        listener.onPlayingIn(null, item);

        WaitForAsyncUtils.waitFor(100, TimeUnit.MILLISECONDS, () -> Objects.equals(component.showName.getText(), title));
        WaitForAsyncUtils.waitFor(100, TimeUnit.MILLISECONDS, () -> component.episodeTitle.getText().equals(caption));
        verify(imageService).load(thumb);
    }

    @Test
    void testOnPlayNextClicked() {
        var event = mock(MouseEvent.class);
        component.initialize(url, resourceBundle);

        component.onPlayNextClicked(event);

        verify(playlistManager).playNext();
    }

    @Test
    void testOnPlayNextPressed() {
        var event = mock(KeyEvent.class);
        when(event.getCode()).thenReturn(KeyCode.ENTER);
        component.initialize(url, resourceBundle);

        component.onPlayNextPressed(event);

        verify(playlistManager).playNext();
    }

    @Test
    void testOnStopClicked() {
        var event = mock(MouseEvent.class);
        component.initialize(url, resourceBundle);

        component.onPlayNextStopClicked(event);

        verify(playlistManager).stop();
    }
}