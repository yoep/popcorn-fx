package com.github.yoep.popcorn.backend.subtitles.parser;

import com.github.yoep.popcorn.backend.subtitles.model.SubtitleCue;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleType;
import lombok.extern.slf4j.Slf4j;

import java.io.ByteArrayInputStream;
import java.io.InputStream;
import java.nio.charset.Charset;
import java.nio.charset.StandardCharsets;
import java.time.Instant;
import java.time.ZoneId;
import java.time.format.DateTimeFormatter;
import java.util.Collections;
import java.util.List;

@Slf4j
public class VttParser implements Parser {
    private static final String HEADER = "WEBVTT";
    private static final String TIME_INDICATOR = " --> ";
    private static final DateTimeFormatter TIME_FORMAT = DateTimeFormatter.ofPattern("HH:mm:ss.SSS");

    @Override
    public boolean support(SubtitleType type) {
        return type == SubtitleType.VTT;
    }

    @Override
    public List<SubtitleCue> parse(InputStream inputStream, Charset encoding) {
        return Collections.emptyList();
    }

    @Override
    public InputStream parse(List<SubtitleCue> cues) {
        var output = new StringBuilder()
                .append(HEADER)
                .append(System.lineSeparator())
                // empty whitespace line
                .append(System.lineSeparator());

        for (SubtitleCue cue : cues) {
            output
                    .append(cue.getId())
                    .append(System.lineSeparator())
                    .append(writeTime(cue))
                    .append(System.lineSeparator());

            cue.getLines().stream()
                    .map(StyleParser::write)
                    .map(e -> e + System.lineSeparator())
                    .forEach(output::append);

            // add an empty line after each cue
            output.append(System.lineSeparator());
        }

        return new ByteArrayInputStream(output.toString().getBytes(StandardCharsets.UTF_8));
    }

    private static String writeTime(SubtitleCue cue) {
        var startTime = Instant.ofEpochMilli(cue.getStartTime()).atZone(ZoneId.of("UTC"));
        var endTime = Instant.ofEpochMilli(cue.getEndTime()).atZone(ZoneId.of("UTC"));

        return TIME_FORMAT.format(startTime) + TIME_INDICATOR + TIME_FORMAT.format(endTime);
    }
}
