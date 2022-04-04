package com.github.yoep.popcorn.backend.subtitles.parser;

import com.github.yoep.popcorn.backend.subtitles.model.SubtitleCue;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleLine;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleText;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleType;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.springframework.core.io.ClassPathResource;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.util.Collections;

import static java.util.Arrays.asList;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class SrtParserTest {
    private SrtParser parser;

    @BeforeEach
    void setUp() {
        parser = new SrtParser();
    }

    @Test
    void testSupports_whenTypeIsSrt_shouldReturnTrue() {
        var result = parser.support(SubtitleType.SRT);

        assertTrue(result, "Expected the parser to support srt format");
    }

    @Test
    void testParse_whenInputIsValid_shouldReturnExpectedResult() throws IOException {
        var resource = new ClassPathResource("subtitles/example.srt");
        var expectedResult = asList(SubtitleCue.builder()
                .id("1")
                .startTime(1200)
                .endTime(3000)
                .lines(Collections.singletonList(new SubtitleLine(Collections.singletonList(new SubtitleText("lorem ipsum dolor", false, false, false)))))
                .build(), SubtitleCue.builder()
                .id("2")
                .startTime(5000)
                .endTime(10000)
                .lines(asList(new SubtitleLine(asList(
                                new SubtitleText("consectetur adipiscing ", false, false, false),
                                new SubtitleText("elit", true, false, false))),
                        new SubtitleLine(Collections.singletonList(new SubtitleText("nec felis dictum mattis", false, false, false)))))
                .build());

        var result = parser.parse(resource.getInputStream(), StandardCharsets.UTF_8);

        assertEquals(expectedResult, result);
    }
}