package com.github.yoep.popcorn.subtitle;

import com.github.yoep.popcorn.subtitle.models.Subtitle;
import com.github.yoep.popcorn.subtitle.models.SubtitleLine;
import org.apache.commons.io.FileUtils;
import org.apache.commons.lang3.StringUtils;
import org.springframework.util.Assert;

import java.io.File;
import java.io.IOException;
import java.nio.charset.Charset;
import java.time.LocalTime;
import java.time.format.DateTimeFormatter;
import java.util.ArrayList;
import java.util.List;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

public class SrtParser {
    private static final Pattern TIME_PATTERN = Pattern.compile("(\\d{1,2}:\\d{2}:\\d{2},\\d{3}) --> (\\d{1,2}:\\d{2}:\\d{2},\\d{3})");
    private static final Pattern TEXT_PATTERN = Pattern.compile("<([a-z])>(.+)</([a-z])>");
    private static final DateTimeFormatter TIME_FORMAT = DateTimeFormatter.ofPattern("HH:mm:ss.SSS");
    private static final String STYLE_ITALIC = "i";
    private static final String STYLE_BOLD = "b";
    private static final String STYLE_UNDERLINE = "u";

    private final List<Subtitle> subtitles = new ArrayList<>();
    private final File file;
    private final Charset charset;

    private Stage stage = Stage.INDEX;
    private Subtitle.SubtitleBuilder subtitleBuilder;
    private List<SubtitleLine> lines;

    private SrtParser(File file, Charset charset) {
        this.file = file;
        this.charset = charset;
    }

    public static List<Subtitle> parse(File file, Charset charset) {
        Assert.notNull(file, "file cannot be null");
        SrtParser parser = new SrtParser(file, charset);

        try {
            return parser.read();
        } catch (IOException ex) {
            throw new SubtitleParsingException("Failed to parse file, " + ex.getMessage(), ex);
        }
    }

    private List<Subtitle> read() throws IOException {
        List<String> lines = FileUtils.readLines(file, charset);

        for (String line : lines) {
            // check if we've reached the end of the current subtitle
            if (StringUtils.isEmpty(line))
                nextStage();

            switch (stage) {
                case INDEX:
                    createNewSubtitle();
                    readIndex(line);
                    break;
                case TIME:
                    readTime(line);
                    break;
                case TEXT:
                    readText(line);
                    break;
                case FINISH:
                    finishSubtitle();
                    break;
            }

        }

        // finish the last subtitle as it might not have been completed
        finishSubtitle();

        return subtitles;
    }

    private void createNewSubtitle() {
        subtitleBuilder = Subtitle.builder();
    }

    private void readIndex(String line) {
        try {
            subtitleBuilder.index(Long.parseLong(line));
        } catch (NumberFormatException ex) {
            throw new SubtitleParsingException("Failed to parse subtitle index, " + ex.getMessage(), ex);
        }

        nextStage();
    }

    private void readTime(String line) {
        Matcher matcher = TIME_PATTERN.matcher(line);

        if (matcher.matches()) {
            subtitleBuilder
                    .startTime(toMillis(LocalTime.parse(matcher.group(1).replace(",", "."), TIME_FORMAT)))
                    .endTime(toMillis(LocalTime.parse(matcher.group(2).replace(",", "."), TIME_FORMAT)));

            nextStage();
        } else {
            throw new SubtitleParsingException("Failed to parse subtitle time, invalid time line format \"" + line + "\"");
        }
    }

    private void readText(String line) {
        if (lines == null)
            lines = new ArrayList<>();

        Matcher matcher = TEXT_PATTERN.matcher(line);

        if (matcher.matches()) {
            String style = matcher.group(1);

            lines.add(SubtitleLine.builder()
                    .text(matcher.group(2))
                    .italic(style.equals(STYLE_ITALIC))
                    .bold(style.equals(STYLE_BOLD))
                    .underline(style.equals(STYLE_UNDERLINE))
                    .build());
        } else {
            lines.add(SubtitleLine.builder()
                    .text(line)
                    .build());
        }
    }

    private void finishSubtitle() {
        if (subtitleBuilder == null)
            return;

        subtitleBuilder.lines(lines);
        subtitles.add(subtitleBuilder.build());
        subtitleBuilder = null;
        lines = null;

        nextStage();
    }

    private long toMillis(LocalTime time) {
        int hour = time.getHour();
        int minutes = (hour * 60) + time.getMinute();
        int seconds = (minutes * 60) + time.getSecond();

        return (seconds * 1000) + (time.getNano() / 1000000);
    }

    private void nextStage() {
        stage = stage.next();
    }

    private enum Stage {
        INDEX,
        TIME,
        TEXT,
        FINISH;

        public Stage next() {
            switch (this) {
                case INDEX:
                    return TIME;
                case TIME:
                    return TEXT;
                case TEXT:
                    return FINISH;
                case FINISH:
                default:
                    return INDEX;
            }
        }
    }
}
