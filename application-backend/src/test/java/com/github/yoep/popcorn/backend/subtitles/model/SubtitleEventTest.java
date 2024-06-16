package com.github.yoep.popcorn.backend.subtitles.model;

import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleLanguage;
import com.github.yoep.popcorn.backend.subtitles.ffi.SubtitleEvent;
import com.github.yoep.popcorn.backend.subtitles.ffi.SubtitleInfo;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class SubtitleEventTest {
    @Test
    void testFrom() {
        var tag = SubtitleEventTag.SubtitleInfoChanged;
        var subtitleInfo = com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo.builder()
                .language(SubtitleLanguage.ENGLISH)
                .files(new SubtitleFile[0])
                .build();
        var ffiEvent = new SubtitleEvent();
        ffiEvent.tag = tag;
        ffiEvent.union = new SubtitleEvent.SubtitleEventCUnion.ByValue();
        ffiEvent.union.subtitle_info_changed = new SubtitleEvent.SubtitleInfoChanged_Body();
        ffiEvent.union.subtitle_info_changed.subtitleInfo = SubtitleInfo.ByReference.from(subtitleInfo);
        var expectedResult = com.github.yoep.popcorn.backend.subtitles.model.SubtitleEvent.builder()
                .tag(tag)
                .subtitleInfo(subtitleInfo)
                .build();

        var result = com.github.yoep.popcorn.backend.subtitles.model.SubtitleEvent.from(ffiEvent);

        assertEquals(expectedResult, result);
    }
}