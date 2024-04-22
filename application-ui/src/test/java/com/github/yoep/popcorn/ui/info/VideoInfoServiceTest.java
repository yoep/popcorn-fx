package com.github.yoep.popcorn.ui.info;

import com.github.yoep.popcorn.backend.adapters.video.VideoPlayback;
import com.github.yoep.popcorn.backend.adapters.video.listeners.VideoListener;
import com.github.yoep.popcorn.backend.adapters.video.state.VideoState;
import com.github.yoep.popcorn.backend.info.ComponentState;
import com.github.yoep.popcorn.backend.info.SimpleComponentDetails;
import org.junit.jupiter.api.Test;

import java.util.Collections;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.Mockito.*;

class VideoInfoServiceTest {
    private VideoInfoService service;

    private final AtomicReference<VideoListener> listenerHolder = new AtomicReference<>();

    @Test
    void testInit_whenInvoked_shouldCreateDetailComponents() {
        var name = "playback-name";
        var description = "playback-description";
        var video = mock(VideoPlayback.class);
        var expectedResult = SimpleComponentDetails.builder()
                .name(name)
                .description(description)
                .state(ComponentState.READY)
                .build();
        when(video.getName()).thenReturn(name);
        when(video.getDescription()).thenReturn(description);
        when(video.getVideoState()).thenReturn(VideoState.READY);

        service = new VideoInfoService(Collections.singletonList(video));

        var result = service.getComponentDetails();
        assertEquals(1, result.size());
        assertEquals(expectedResult, result.get(0));
    }

    @Test
    void testListener_whenVideoStateIsChanged_shouldUpdateComponentState() {
        var name = "playback-name";
        var description = "playback-description";
        var video = mock(VideoPlayback.class);
        when(video.getName()).thenReturn(name);
        when(video.getDescription()).thenReturn(description);
        when(video.getVideoState()).thenReturn(VideoState.READY);
        doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(0, VideoListener.class));
            return null;
        }).when(video).addListener(isA(VideoListener.class));
        service = new VideoInfoService(Collections.singletonList(video));

        var listener = listenerHolder.get();
        listener.onStateChanged(VideoState.ERROR);

        var result = service.getComponentDetails();
        assertEquals(1, result.size());
        assertEquals(ComponentState.ERROR, result.get(0).getState());
    }
}