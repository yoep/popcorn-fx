package com.github.yoep.popcorn.backend.subtitles.ffi;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class SubtitleFileTest {
    @Test
    void testByReferenceFrom() {
        var file = com.github.yoep.popcorn.backend.subtitles.model.SubtitleFile.builder()
                .fileId(1)
                .name("name")
                .url("url")
                .score(1)
                .downloads(1)
                .quality(null)
                .build();

        var result = SubtitleFile.ByReference.from(file);

        assertEquals(1, result.fileId);
        assertEquals("name", result.name);
        assertEquals("url", result.url);
        assertEquals(1, result.score);
        assertEquals(1, result.downloads);
        assertNull(result.quality);
    }
}