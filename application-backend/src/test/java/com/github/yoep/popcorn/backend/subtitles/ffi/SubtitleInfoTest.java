package com.github.yoep.popcorn.backend.subtitles.ffi;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleLanguage;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleFile;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;

@ExtendWith(MockitoExtension.class)
class SubtitleInfoTest {
    @Mock
    private FxLib fxLib;

    @BeforeEach
    void setUp() {
        FxLib.INSTANCE.set(fxLib);
    }

    @Test
    void testNewReferenceInstance_ImdbAndLanguage() {
        var imdbId = "tt123764551";
        var language = SubtitleLanguage.DANISH;
        var info = new SubtitleInfo.ByReference(imdbId, language);

        assertEquals(imdbId, info.imdbId);
        assertEquals(language, info.language);
    }

    @Test
    void testNewReferenceFrom() {
        var imdbId = "tt000123";
        var language = SubtitleLanguage.FRENCH;
        var modelInfo = com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo.builder()
                .imdbId(imdbId)
                .language(language)
                .files(new com.github.yoep.popcorn.backend.subtitles.model.SubtitleFile[]{
                        SubtitleFile.builder()
                                .fileId(1)
                                .name("FooBar")
                                .url("http://example.com/subtitle.srt")
                                .build()
                })
                .build();
        var info = SubtitleInfo.ByReference.from(modelInfo);

        assertEquals(imdbId, info.imdbId);
        assertEquals(language, info.language);
        assertNotNull(info.files, "Subtitle files should not be null");
        assertEquals(1, info.len);
    }

    @Test
    void testClose() {
        var info = new SubtitleInfo.ByReference("imdbId", SubtitleLanguage.ENGLISH);

        info.close();
    }
}