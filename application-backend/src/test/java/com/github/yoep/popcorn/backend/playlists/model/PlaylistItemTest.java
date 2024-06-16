package com.github.yoep.popcorn.backend.playlists.model;

import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentFileInfo;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentInfo;
import com.github.yoep.popcorn.backend.playlists.ffi.PlaylistItem;
import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleLanguage;
import com.github.yoep.popcorn.backend.subtitles.ffi.SubtitleInfo;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleFile;
import org.junit.jupiter.api.Test;

import java.util.Collections;

import static org.junit.jupiter.api.Assertions.*;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

class PlaylistItemTest {

    @Test
    void testFrom_withoutOptionals() {
        var url = "http://localhost/example-video.mp4";
        var title = "Example Video";
        var thumb = "http://localhost/example-thumb.png";
        var autoResumeTimestamp = 1000L;
        var subtitlesEnabled = true;
        var ffiItem = PlaylistItem.builder()
                .url(url)
                .title(title)
                .thumb(thumb)
                .autoResumeTimestamp(autoResumeTimestamp)
                .subtitlesEnabled(subtitlesEnabled)
                .build();
        var expectedResult = com.github.yoep.popcorn.backend.playlists.model.PlaylistItem.builder()
                .url(url)
                .title(title)
                .thumb(thumb)
                .autoResumeTimestamp(autoResumeTimestamp)
                .subtitlesEnabled(subtitlesEnabled)
                .build();

        var result = com.github.yoep.popcorn.backend.playlists.model.PlaylistItem.from(ffiItem);

        assertEquals(expectedResult, result);
    }

    @Test
    void testFrom_withOptionals() {
        var url = "http://localhost/example-video.mp4";
        var title = "Example Video";
        var thumb = "http://localhost/example-thumb.png";
        var subtitleInfo = com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo.builder()
                .language(SubtitleLanguage.ENGLISH)
                .files(new SubtitleFile[0])
                .build();
        var torrentInfo = mock(TorrentInfo.class);
        var torrentFileInfo = mock(TorrentFileInfo.class);
        when(torrentInfo.getFiles()).thenReturn(Collections.singletonList(torrentFileInfo));
        var ffiItem = PlaylistItem.builder()
                .url(url)
                .title(title)
                .thumb(thumb)
                .autoResumeTimestamp(1000L)
                .subtitlesEnabled(true)
                .subtitleInfo(SubtitleInfo.ByReference.from(subtitleInfo))
                .torrentInfo(torrentInfo)
                .torrentFileInfo(torrentFileInfo)
                .build();
        var expectedResult = com.github.yoep.popcorn.backend.playlists.model.PlaylistItem.builder()
                .url(url)
                .title(title)
                .thumb(thumb)
                .autoResumeTimestamp(1000L)
                .subtitlesEnabled(true)
                .subtitleInfo(subtitleInfo)
                .torrentInfo(ffiItem.getTorrentInfo())
                .torrentFileInfo(ffiItem.getTorrentFileInfo())
                .build();

        var result = com.github.yoep.popcorn.backend.playlists.model.PlaylistItem.from(ffiItem);

        assertEquals(expectedResult, result);
    }
}