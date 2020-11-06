package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.ui.events.ClosePlayerEvent;
import com.github.yoep.popcorn.ui.events.PlayerStoppedEvent;
import com.github.yoep.popcorn.ui.media.resume.AutoResumeService;
import com.github.yoep.popcorn.ui.settings.SettingsService;
import com.github.yoep.torrent.adapter.TorrentStreamService;
import org.junit.jupiter.api.Nested;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.context.ApplicationEventPublisher;

import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.verify;

@ExtendWith(MockitoExtension.class)
public class VideoPlayerServiceTest {
    @Mock
    private ApplicationEventPublisher eventPublisher;
    @Mock
    private AutoResumeService autoResumeService;
    @Mock
    private FullscreenService fullscreenService;
    @Mock
    private TorrentStreamService torrentStreamService;
    @Mock
    private SettingsService settingsService;
    @Mock
    private VideoPlayerManagerService videoPlayerManagerService;
    @Mock
    private VideoPlayerSubtitleService videoPlayerSubtitleService;
    @InjectMocks
    private VideoPlayerService videoPlayerService;

    @Nested
    class Listeners {
        @Test
        void testAddListener_whenListenerIsNull_shouldThrowIllegalArgumentException() {
            assertThrows(IllegalArgumentException.class, () -> videoPlayerService.addListener(null), "listener cannot be null");
        }
    }

    @Nested
    class Stop {
        @Test
        void testStop_whenInvoked_shouldPublishPlayerStoppedEvent() {
            videoPlayerService.stop();

            verify(eventPublisher).publishEvent(isA(PlayerStoppedEvent.class));
        }

        @Test
        void testStop_whenInvoked_shouldStopTorrentStream() {
            videoPlayerService.stop();

            verify(torrentStreamService).stopAllStreams();
        }
    }

    @Nested
    class Close {
        @Test
        void testClose_whenInvoked_shouldPublishClosePlayerEvent() {
            videoPlayerService.close();

            verify(eventPublisher).publishEvent(new ClosePlayerEvent(videoPlayerService));
        }
    }
}
