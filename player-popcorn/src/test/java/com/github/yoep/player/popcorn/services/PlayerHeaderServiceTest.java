package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.popcorn.listeners.PlaybackListener;
import com.github.yoep.player.popcorn.listeners.PlayerHeaderListener;
import com.github.yoep.player.popcorn.player.PopcornPlayer;
import com.github.yoep.popcorn.backend.player.model.MediaPlayRequest;
import com.github.yoep.popcorn.backend.player.model.SimplePlayRequest;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.concurrent.atomic.AtomicReference;

import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.lenient;
import static org.mockito.Mockito.verify;

@ExtendWith(MockitoExtension.class)
class PlayerHeaderServiceTest {
    @Mock
    private PopcornPlayer player;
    @Mock
    private VideoService videoService;
    @Mock
    private PlayerHeaderListener listener;
    @InjectMocks
    private PlayerHeaderService service;

    private final AtomicReference<PlaybackListener> listenerHolder = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        lenient().doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(0, PlaybackListener.class));
            return null;
        }).when(videoService).addListener(isA(PlaybackListener.class));

        service.addListener(listener);
    }

    @Test
    void testStop_whenInvokeD_shouldStopThePlayer() {
        service.stop();

        verify(player).stop();
    }

    @Test
    void testPlaybackListener_whenPlayRequestInvoked_shouldSetTheTitle() {
        var expectedTitle = "lorem ipsum dolor";
        var request = SimplePlayRequest.builder()
                .title(expectedTitle)
                .build();
        service.init();

        listenerHolder.get().onPlay(request);

        verify(listener).onTitleChanged(expectedTitle);
    }

    @Test
    void testPlaybackListener_whenRequestIsMediaPlayRequest_shouldSetTheQuality() {
        var expectedQuality = "1080p";
        var request = MediaPlayRequest.mediaBuilder()
                .quality(expectedQuality)
                .build();
        service.init();

        listenerHolder.get().onPlay(request);

        verify(listener).onQualityChanged(expectedQuality);
    }
}