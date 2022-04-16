package com.github.yoep.popcorn.backend.subtitles.parser;

import com.github.yoep.popcorn.backend.subtitles.model.SubtitleLine;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleText;
import org.junit.jupiter.api.Test;

import java.util.Collections;

import static org.junit.jupiter.api.Assertions.assertEquals;

class StyleParserTest {
    @Test
    void testParseLineStyle_whenStyleContainsItalic_shouldReturnItalicStyle() {
        var line = "<i>lorem</i>";
        var expectedResult = SubtitleLine.builder()
                .texts(Collections.singletonList(SubtitleText.builder()
                        .text("lorem")
                        .italic(true)
                        .build()))
                .build();

        var result = StyleParser.parseLineStyle(line);

        assertEquals(expectedResult, result);
    }

    @Test
    void testParseLineStyle_whenStyleContainsBold_shouldReturnBoldStyle() {
        var line = "<b>lorem</b>";
        var expectedResult = SubtitleLine.builder()
                .texts(Collections.singletonList(SubtitleText.builder()
                        .text("lorem")
                        .bold(true)
                        .build()))
                .build();

        var result = StyleParser.parseLineStyle(line);

        assertEquals(expectedResult, result);
    }

    @Test
    void testParseLineStyle_whenStyleContainsUnderline_shouldReturnUnderlineStyle() {
        var line = "<u>lorem</u>";
        var expectedResult = SubtitleLine.builder()
                .texts(Collections.singletonList(SubtitleText.builder()
                        .text("lorem")
                        .underline(true)
                        .build()))
                .build();

        var result = StyleParser.parseLineStyle(line);

        assertEquals(expectedResult, result);
    }

    @Test
    void testWrite_whenSubtitleTextHasItalicStyle_shouldWriteItalicTag() {
        var line = SubtitleLine.builder()
                .texts(Collections.singletonList(SubtitleText.builder()
                        .text("ipsum")
                        .italic(true)
                        .build()))
                .build();
        var expectedResult = "<i>ipsum</i>";

        var result = StyleParser.write(line);

        assertEquals(expectedResult, result);
    }

    @Test
    void testWrite_whenSubtitleTextHasBoldStyle_shouldWriteBoldTag() {
        var line = SubtitleLine.builder()
                .texts(Collections.singletonList(SubtitleText.builder()
                        .text("ipsum")
                        .bold(true)
                        .build()))
                .build();
        var expectedResult = "<b>ipsum</b>";

        var result = StyleParser.write(line);

        assertEquals(expectedResult, result);
    }

    @Test
    void testWrite_whenSubtitleTextHasUnderlineStyle_shouldWriteUnderlineTag() {
        var line = SubtitleLine.builder()
                .texts(Collections.singletonList(SubtitleText.builder()
                        .text("ipsum")
                        .underline(true)
                        .build()))
                .build();
        var expectedResult = "<u>ipsum</u>";

        var result = StyleParser.write(line);

        assertEquals(expectedResult, result);
    }

    @Test
    void testWrite_whenSubtitleTextHasNoStyle_shouldNotAddAnyTag() {
        var line = SubtitleLine.builder()
                .texts(Collections.singletonList(SubtitleText.builder()
                        .text("estla")
                        .build()))
                .build();
        var expectedResult = "estla";

        var result = StyleParser.write(line);

        assertEquals(expectedResult, result);
    }
}