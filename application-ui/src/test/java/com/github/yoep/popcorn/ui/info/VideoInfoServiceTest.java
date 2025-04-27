package com.github.yoep.popcorn.ui.info;

import com.github.yoep.popcorn.backend.adapters.video.VideoPlayback;
import com.github.yoep.popcorn.backend.adapters.video.listeners.VideoListener;
import com.github.yoep.popcorn.backend.adapters.video.state.VideoState;
import com.github.yoep.popcorn.backend.info.ComponentState;
import com.github.yoep.popcorn.backend.info.SimpleComponentDetails;
import com.github.yoep.popcorn.ui.IoC;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.Collections;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class VideoInfoServiceTest {
    @Mock
    private IoC ioc;
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
        when(ioc.getInstances(VideoPlayback.class)).thenReturn(Collections.singletonList(video));

        service = new VideoInfoService(ioc);

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
        when(ioc.getInstances(VideoPlayback.class)).thenReturn(Collections.singletonList(video));
        service = new VideoInfoService(ioc);

        var componentDetails = service.getComponentDetails();

        var listener = listenerHolder.get();
        listener.onStateChanged(VideoState.ERROR);

        assertEquals(1, componentDetails.size());
        assertEquals(ComponentState.ERROR, componentDetails.getFirst().getState());
    }
}