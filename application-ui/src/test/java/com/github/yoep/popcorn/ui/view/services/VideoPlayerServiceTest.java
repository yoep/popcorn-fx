package com.github.yoep.popcorn.ui.view.services;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.ui.events.ClosePlayerEvent;
import com.github.yoep.popcorn.ui.events.PlayerStoppedEvent;
import com.github.yoep.popcorn.ui.media.resume.AutoResumeService;
import com.github.yoep.popcorn.ui.settings.SettingsService;
import com.github.yoep.popcorn.ui.subtitles.Subtitle;
import com.github.yoep.popcorn.ui.subtitles.SubtitlePickerService;
import com.github.yoep.popcorn.ui.subtitles.SubtitleService;
import com.github.yoep.popcorn.ui.subtitles.models.SubtitleFile;
import com.github.yoep.popcorn.ui.subtitles.models.SubtitleInfo;
import com.github.yoep.popcorn.ui.subtitles.models.SubtitleLanguage;
import com.github.yoep.popcorn.ui.subtitles.models.SubtitleMatcher;
import com.github.yoep.torrent.adapter.TorrentStreamService;
import com.github.yoep.video.adapter.VideoPlayer;
import org.junit.jupiter.api.Nested;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.context.ApplicationEventPublisher;

import java.util.List;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.mockito.ArgumentMatchers.*;
import static org.mockito.Mockito.*;

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
    private SubtitleService subtitleService;
    @Mock
    private SubtitlePickerService subtitlePickerService;
    @Mock
    private LocaleText localeText;
    @Mock
    private List<VideoPlayer> videoPlayers;
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
    class SetSubtitle {

        @Test
        public void testSetSubtitle_whenSubtitleInfoIsNull_shouldSetSubtitlePropertyToNone() {
            var expectedResult = Subtitle.none();

            videoPlayerService.setSubtitle((SubtitleInfo) null);
            var result = videoPlayerService.getSubtitle();

            assertEquals(expectedResult, result);
        }

        @Test
        public void testSetSubtitle_whenSubtitleInfoIsNone_shouldSetSubtitlePropertyToNone() {
            var expectedResult = Subtitle.none();

            videoPlayerService.setSubtitle(SubtitleInfo.none());
            var result = videoPlayerService.getSubtitle();

            assertEquals(expectedResult, result);
        }

        @Test
        public void testSetSubtitle_whenIsNotNullOrNone_shouldCallDownloadAndParseOnTheSubtitleService() {
            var subtitleInfo = new SubtitleInfo("db0001", SubtitleLanguage.ENGLISH);
            when(subtitleService.downloadAndParse(any(), any())).thenReturn(new CompletableFuture<>());

            videoPlayerService.setSubtitle(subtitleInfo);

            verify(subtitleService).downloadAndParse(eq(subtitleInfo), isA(SubtitleMatcher.class));
        }

        @Test
        public void testSetSubtitle_whenSubtitleIsCustom_shouldLetTheUserPickASubtitleFile() {
            videoPlayerService.setSubtitle(SubtitleInfo.custom());

            verify(subtitlePickerService).pickCustomSubtitle();
        }

        @Test
        public void testSetSubtitle_whenSubtitleIsCustomAndContainsFile_shouldNotLetTheUserPickASubtitleFile() {
            var customSubtitle = SubtitleInfo.custom();
            customSubtitle.addFile(mock(SubtitleFile.class));
            when(subtitleService.downloadAndParse(eq(customSubtitle), isA(SubtitleMatcher.class)))
                    .thenReturn(CompletableFuture.completedFuture(mock(Subtitle.class)));

            videoPlayerService.setSubtitle(customSubtitle);

            verify(subtitlePickerService, times(0)).pickCustomSubtitle();
        }

        @Test
        public void testSetSubtitle_whenSubtitleIsCustomAndUserCancelled_shouldDisableTheSubtitle() {
            var expectedResult = Subtitle.none();
            when(subtitlePickerService.pickCustomSubtitle()).thenReturn(Optional.empty());

            videoPlayerService.setSubtitle(SubtitleInfo.custom());
            var result = videoPlayerService.getSubtitle();

            assertEquals(expectedResult, result);
        }

        @Test
        public void testSetSubtitle_whenSubtitleIsCustomAndUserPickedSubtitleFile_shouldUpdateSubtitleTrackWithCustomSubtitleFile() {
            var customSubtitle = SubtitleInfo.custom();
            var expectedResult = mock(Subtitle.class);
            when(subtitlePickerService.pickCustomSubtitle()).thenReturn(Optional.of(customSubtitle));
            when(subtitleService.downloadAndParse(eq(customSubtitle), isA(SubtitleMatcher.class))).thenReturn(CompletableFuture.completedFuture(expectedResult));

            videoPlayerService.setSubtitle(SubtitleInfo.custom());
            var result = videoPlayerService.getSubtitle();

            assertEquals(expectedResult, result);
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
