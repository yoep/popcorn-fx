package com.github.yoep.popcorn.backend.subtitles.model;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleLanguage;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.mockito.Mockito.verify;

@ExtendWith(MockitoExtension.class)
class SubtitleInfoTest {
    @Mock
    private FxLib fxLib;

    @BeforeEach
    void setUp() {
         FxLib.INSTANCE.set(fxLib);
    }

    @Test
    void testClose() {
        var info = new SubtitleInfo.ByReference("imdbId", SubtitleLanguage.ENGLISH);

        info.close();

        verify(fxLib).dispose_subtitle_info(info);
    }
}