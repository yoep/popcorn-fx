package com.github.yoep.player.popcorn.controllers.sections;

import com.github.yoep.player.popcorn.services.PlaybackService;
import javafx.collections.ObservableList;
import javafx.scene.Node;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.Pane;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.mockito.Mockito.*;


@ExtendWith(MockitoExtension.class)
class PopcornPlayerSectionControllerTest {
    @Mock
    private PlaybackService playbackService;
    @Mock
    private ObservableList<Node> items;
    @InjectMocks
    private PopcornPlayerSectionController controller;

    @Test
    void testSetVideoView_whenInvoked_shouldSwitchTheVideoView() {
        var videoView = mock(Pane.class);
        var view = mock(Node.class);
        when(videoView.getChildren()).thenReturn(items);

        controller.videoView = videoView;
        controller.setVideoView(view);

        verify(items).setAll(view);
    }

    @Test
    void testOnPlayerClick_whenInvoked_shouldConsumeTheMouseEvent() {
        var event = mock(MouseEvent.class);

        controller.onPlayerClick(event);

        verify(event).consume();
    }

    @Test
    void testOnPlayerClick_whenInvoked_shouldToggleThePlayPauseOfThePlayer() {
        var event = mock(MouseEvent.class);

        controller.onPlayerClick(event);

        verify(playbackService).togglePlayerPlaybackState();
    }
}
