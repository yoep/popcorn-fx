package com.github.yoep.popcorn.backend.subtitles.parser;

import com.github.yoep.popcorn.backend.subtitles.model.SubtitleCue;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleLine;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleText;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleType;
import org.apache.commons.io.IOUtils;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Disabled;
import org.junit.jupiter.api.Test;
import org.springframework.core.io.ClassPathResource;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.util.Collections;

import static java.util.Arrays.asList;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class VttParserTest {
    private VttParser parser;

    @BeforeEach
    void setUp() {
        parser = new VttParser();
    }

    @Test
    void testSupports_whenTypeIsSrt_shouldReturnTrue() {
        var result = parser.support(SubtitleType.VTT);

        assertTrue(result, "Expected the parser to support vtt format");
    }

    // TODO: implement actual code
    @Test
    @Disabled
    void testParse_whenInputIsValid_shouldReturnExpectedCues() throws IOException {
        var resource = new ClassPathResource("subtitles/example.vtt");
        var expectedResult = asList(SubtitleCue.builder()
                .id("1")
                .startTime(1000)
                .endTime(3000)
                .lines(Collections.singletonList(new SubtitleLine(Collections.singletonList(new SubtitleText("lorem ipsum dolor", false, false, false)))))
                .build(), SubtitleCue.builder()
                .id("2")
                .startTime(5000)
                .endTime(10000)
                .lines(asList(new SubtitleLine(asList(
                                new SubtitleText("consectetur adipiscing ", false, false, false),
                                new SubtitleText("elit", false, false, false))),
                        new SubtitleLine(Collections.singletonList(new SubtitleText("nec felis dictum mattis", false, false, false)))))
                .build());

        var result = parser.parse(resource.getInputStream(), StandardCharsets.UTF_8);

        assertEquals(expectedResult, result);
    }

    @Test
    void testParse_whenSubtitleCuesISGiven_shouldReturnExpectedInputStream() throws IOException {
        var resource = new ClassPathResource("subtitles/parse-to-inputstream.vtt");
        var line1 = new SubtitleLine(Collections.singletonList(new SubtitleText("lorem ipsum dolor", false, false, false)));
        var line2 = new SubtitleLine(Collections.singletonList(new SubtitleText("estla", false, false, false)));
        var cues = Collections.singletonList(SubtitleCue.builder()
                .id("1")
                .startTime(0)
                .endTime(5000)
                .lines(asList(line1, line2))
                .build());
        var expectedResult = IOUtils.toString(resource.getInputStream(), StandardCharsets.UTF_8);

        var output = parser.parse(cues);
        var result = IOUtils.toString(output, StandardCharsets.UTF_8);

        assertEquals(expectedResult.trim(), result.trim());
    }
}