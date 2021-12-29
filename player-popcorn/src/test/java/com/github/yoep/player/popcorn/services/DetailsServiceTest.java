package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.popcorn.controllers.components.PlayerHeaderComponent;
import com.github.yoep.player.popcorn.controllers.sections.PopcornPlayerSectionController;
import com.github.yoep.popcorn.backend.adapters.video.VideoPlayer;
import javafx.beans.property.SimpleObjectProperty;
import javafx.scene.Node;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class DetailsServiceTest {
    @Mock
    private VideoService videoService;
    @Mock
    private PopcornPlayerSectionController playerSectionController;
    @Mock
    private PlayerHeaderComponent playerHeaderComponent;
    @InjectMocks
    private DetailsService service;

    @Test
    void testInit_whenVideoPlayerIsSwitched_shouldUpdateVideoSurface() {
        var newPlayer = mock(VideoPlayer.class);
        var newPlayerVideoView = mock(Node.class);
        var videoPlayerProperty = new SimpleObjectProperty<VideoPlayer>();
        when(videoService.videoPlayerProperty()).thenReturn(videoPlayerProperty);
        when(newPlayer.getVideoSurface()).thenReturn(newPlayerVideoView);

        service.init();
        videoPlayerProperty.set(newPlayer);

        verify(playerSectionController).setVideoView(newPlayerVideoView);
    }
}
