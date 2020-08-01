package com.github.yoep.popcorn.ui.view.services;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.ui.activities.ActivityManager;
import com.github.yoep.popcorn.ui.media.resume.AutoResumeService;
import com.github.yoep.popcorn.ui.settings.SettingsService;
import com.github.yoep.popcorn.ui.subtitles.Subtitle;
import com.github.yoep.popcorn.ui.subtitles.SubtitleService;
import com.github.yoep.popcorn.ui.subtitles.models.SubtitleInfo;
import com.github.yoep.popcorn.ui.subtitles.models.SubtitleLanguage;
import com.github.yoep.popcorn.ui.subtitles.models.SubtitleMatcher;
import com.github.yoep.torrent.adapter.TorrentStreamService;
import com.github.yoep.video.adapter.VideoPlayer;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import java.util.List;
import java.util.concurrent.CompletableFuture;

import static org.junit.Assert.assertEquals;
import static org.mockito.ArgumentMatchers.*;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.class)
public class VideoPlayerServiceTest {
    @Mock
    private ActivityManager activityManager;
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
    private LocaleText localeText;
    @Mock
    private List<VideoPlayer> videoPlayers;
    @InjectMocks
    private VideoPlayerService videoPlayerService;

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
}
