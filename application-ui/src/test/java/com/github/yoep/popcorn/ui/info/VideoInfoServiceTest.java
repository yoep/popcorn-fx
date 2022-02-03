package com.github.yoep.popcorn.ui.info;

import com.github.yoep.popcorn.backend.adapters.video.VideoPlayback;
import com.github.yoep.popcorn.backend.adapters.video.state.VideoState;
import com.github.yoep.popcorn.backend.info.ComponentState;
import com.github.yoep.popcorn.backend.info.SimpleComponentDetails;
import org.junit.jupiter.api.Test;

import java.util.Collections;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

class VideoInfoServiceTest {
    private VideoInfoService service;

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

        service.init();

        var result = service.getComponentDetails();
        assertEquals(1, result.size());
        assertEquals(expectedResult, result.get(0));
    }
}