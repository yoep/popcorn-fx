package com.github.yoep.popcorn.backend.subtitles.parser;

import com.github.yoep.popcorn.backend.subtitles.SubtitleParsingException;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleCue;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleLine;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleType;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.lang3.StringUtils;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.nio.charset.Charset;
import java.text.MessageFormat;
import java.time.LocalTime;
import java.time.format.DateTimeFormatter;
import java.util.ArrayList;
import java.util.List;
import java.util.Objects;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

@Slf4j
public class SrtParser implements Parser {
    private static final Pattern TIME_PATTERN = Pattern.compile("(\\d{1,2}:\\d{2}:\\d{2},\\d{3}) --> (\\d{1,2}:\\d{2}:\\d{2},\\d{3})");
    private static final DateTimeFormatter TIME_FORMAT = DateTimeFormatter.ofPattern("HH:mm:ss.SSS");

    @Override
    public boolean support(SubtitleType type) {
        return type == SubtitleType.SRT;
    }

    @Override
    public List<SubtitleCue> parse(InputStream inputStream, Charset encoding) {
        Objects.requireNonNull(inputStream, "inputStream cannot be null");
        Objects.requireNonNull(encoding, "encoding cannot be null");
        var stage = Stage.IDENTIFIER;
        var cueBuilder = SubtitleCue.builder();
        var lineIndex = 0;
        var line = "";
        var lines = new ArrayList<SubtitleLine>();
        var cues = new ArrayList<SubtitleCue>();
        var reader = new BufferedReader(new InputStreamReader(inputStream, encoding));

        try {
            while ((line = reader.readLine()) != null) {
                // check if we've reached the end of the current cue
                if (StringUtils.isEmpty(line))
                    stage = stage.next();

                try {
                    switch (stage) {
                        case IDENTIFIER -> {
                            cueBuilder = readIdentifier(line);
                            lines = new ArrayList<>();
                            stage = stage.next();
                        }
                        case TIME -> {
                            readTime(cueBuilder, lineIndex, line);
                            stage = stage.next();
                        }
                        case TEXT -> lines.add(StyleParser.parseLineStyle(line));
                        case FINISH -> {
                            cues.add(finishSubtitle(cueBuilder, lines));
                            stage = stage.next();
                        }
                    }
                } catch (SubtitleParsingException ex) {
                    var message = MessageFormat.format("Subtitle line {0} is invalid and will be ignored", lineIndex);
                    log.warn(message, ex);
                }
            }
        } catch (IOException ex) {
            throw new SubtitleParsingException(ex.getMessage(), ex);
        }

        // finish the last subtitle as it might not have been completed
        cues.add(finishSubtitle(cueBuilder, lines));

        return cues;
    }

    @Override
    public InputStream parse(List<SubtitleCue> cues) {
        return null;
    }

    private SubtitleCue.SubtitleCueBuilder readIdentifier(String line) {
        var cueBuilder = SubtitleCue.builder();
        cueBuilder.id(line);

        return cueBuilder;
    }

    private void readTime(SubtitleCue.SubtitleCueBuilder cueBuilder, int lineIndex, String line) {
        Matcher matcher = TIME_PATTERN.matcher(line);

        if (matcher.matches()) {
            cueBuilder
                    .startTime(toMillis(LocalTime.parse(matcher.group(1).replace(",", "."), TIME_FORMAT)))
                    .endTime(toMillis(LocalTime.parse(matcher.group(2).replace(",", "."), TIME_FORMAT)));
        } else {
            String message = MessageFormat.format("Failed to parse subtitle time at line {0}, invalid time line format \"{1}\"", lineIndex, line);
            throw new SubtitleParsingException(message);
        }
    }

    private SubtitleCue finishSubtitle(SubtitleCue.SubtitleCueBuilder cueBuilder, ArrayList<SubtitleLine> lines) {
        if (cueBuilder == null)
            return null;

        cueBuilder.lines(lines);

        return cueBuilder.build();
    }

    private long toMillis(LocalTime time) {
        int hour = time.getHour();
        int minutes = (hour * 60) + time.getMinute();
        int seconds = (minutes * 60) + time.getSecond();

        return (seconds * 1000) + (long) (time.getNano() / 1000000);
    }

    private enum Stage {
        IDENTIFIER,
        TIME,
        TEXT,
        FINISH;

        public Stage next() {
            return switch (this) {
                case IDENTIFIER -> TIME;
                case TIME -> TEXT;
                case TEXT -> FINISH;
                default -> IDENTIFIER;
            };
        }
    }
}
