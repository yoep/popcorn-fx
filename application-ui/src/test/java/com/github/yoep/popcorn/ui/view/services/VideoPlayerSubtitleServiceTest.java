package com.github.yoep.popcorn.ui.view.services;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.ui.settings.SettingsService;
import com.github.yoep.popcorn.ui.subtitles.Subtitle;
import com.github.yoep.popcorn.ui.subtitles.SubtitlePickerService;
import com.github.yoep.popcorn.ui.subtitles.SubtitleService;
import com.github.yoep.popcorn.ui.subtitles.models.SubtitleFile;
import com.github.yoep.popcorn.ui.subtitles.models.SubtitleInfo;
import com.github.yoep.popcorn.ui.subtitles.models.SubtitleLanguage;
import com.github.yoep.popcorn.ui.subtitles.models.SubtitleMatcher;
import org.junit.jupiter.api.Nested;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.context.ApplicationEventPublisher;

import java.util.Optional;
import java.util.concurrent.CompletableFuture;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.ArgumentMatchers.*;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class VideoPlayerSubtitleServiceTest {
    @Mock
    private SettingsService settingsService;
    @Mock
    private SubtitleService subtitleService;
    @Mock
    private SubtitlePickerService subtitlePickerService;
    @Mock
    private ApplicationEventPublisher eventPublisher;
    @Mock
    private LocaleText localeText;
    @InjectMocks
    private VideoPlayerSubtitleService videoPlayerSubtitleService;

    @Nested
    class SetSubtitle {

        @Test
        public void testSetSubtitle_whenSubtitleInfoIsNull_shouldSetSubtitlePropertyToNone() {
            var expectedResult = Subtitle.none();

            videoPlayerSubtitleService.setSubtitle((SubtitleInfo) null);
            var result = videoPlayerSubtitleService.getSubtitle();

            assertEquals(expectedResult, result);
        }

        @Test
        public void testSetSubtitle_whenSubtitleInfoIsNone_shouldSetSubtitlePropertyToNone() {
            var expectedResult = Subtitle.none();

            videoPlayerSubtitleService.setSubtitle(SubtitleInfo.none());
            var result = videoPlayerSubtitleService.getSubtitle();

            assertEquals(expectedResult, result);
        }

        @Test
        public void testSetSubtitle_whenIsNotNullOrNone_shouldCallDownloadAndParseOnTheSubtitleService() {
            var subtitleInfo = new SubtitleInfo("db0001", SubtitleLanguage.ENGLISH);
            when(subtitleService.downloadAndParse(any(), any())).thenReturn(new CompletableFuture<>());

            videoPlayerSubtitleService.setSubtitle(subtitleInfo);

            verify(subtitleService).downloadAndParse(eq(subtitleInfo), isA(SubtitleMatcher.class));
        }

        @Test
        public void testSetSubtitle_whenSubtitleIsCustom_shouldLetTheUserPickASubtitleFile() {
            videoPlayerSubtitleService.setSubtitle(SubtitleInfo.custom());

            verify(subtitlePickerService).pickCustomSubtitle();
        }

        @Test
        public void testSetSubtitle_whenSubtitleIsCustomAndContainsFile_shouldNotLetTheUserPickASubtitleFile() {
            var customSubtitle = SubtitleInfo.custom();
            customSubtitle.addFile(mock(SubtitleFile.class));
            when(subtitleService.downloadAndParse(eq(customSubtitle), isA(SubtitleMatcher.class)))
                    .thenReturn(CompletableFuture.completedFuture(mock(Subtitle.class)));

            videoPlayerSubtitleService.setSubtitle(customSubtitle);

            verify(subtitlePickerService, times(0)).pickCustomSubtitle();
        }

        @Test
        public void testSetSubtitle_whenSubtitleIsCustomAndUserCancelled_shouldDisableTheSubtitle() {
            var expectedResult = Subtitle.none();
            when(subtitlePickerService.pickCustomSubtitle()).thenReturn(Optional.empty());

            videoPlayerSubtitleService.setSubtitle(SubtitleInfo.custom());
            var result = videoPlayerSubtitleService.getSubtitle();

            assertEquals(expectedResult, result);
        }

        @Test
        public void testSetSubtitle_whenSubtitleIsCustomAndUserPickedSubtitleFile_shouldUpdateSubtitleTrackWithCustomSubtitleFile() {
            var customSubtitle = SubtitleInfo.custom();
            var expectedResult = mock(Subtitle.class);
            when(subtitlePickerService.pickCustomSubtitle()).thenReturn(Optional.of(customSubtitle));
            when(subtitleService.downloadAndParse(eq(customSubtitle), isA(SubtitleMatcher.class))).thenReturn(CompletableFuture.completedFuture(expectedResult));

            videoPlayerSubtitleService.setSubtitle(SubtitleInfo.custom());
            var result = videoPlayerSubtitleService.getSubtitle();

            assertEquals(expectedResult, result);
        }
    }
}
