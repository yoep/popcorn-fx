package com.github.yoep.popcorn.ui.subtitles;

import com.github.yoep.popcorn.ui.subtitles.models.SubtitleIndex;
import com.github.yoep.popcorn.ui.subtitles.models.SubtitleLine;
import com.github.yoep.popcorn.ui.subtitles.models.SubtitleText;
import org.apache.commons.io.FileUtils;
import org.apache.commons.lang3.StringUtils;
import org.springframework.util.Assert;

import java.io.File;
import java.io.IOException;
import java.nio.charset.Charset;
import java.text.MessageFormat;
import java.time.LocalTime;
import java.time.format.DateTimeFormatter;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

public class SrtParser {
    private static final Pattern INDEX_PATTERN = Pattern.compile("([0-9]+)");
    private static final Pattern TIME_PATTERN = Pattern.compile("(\\d{1,2}:\\d{2}:\\d{2},\\d{3}) --> (\\d{1,2}:\\d{2}:\\d{2},\\d{3})");
    private static final Pattern TEXT_PATTERN = Pattern.compile("(<([a-z])>)?([^<]+)(</([a-z])>)?");
    private static final DateTimeFormatter TIME_FORMAT = DateTimeFormatter.ofPattern("HH:mm:ss.SSS");
    private static final String STYLE_ITALIC = "i";
    private static final String STYLE_BOLD = "b";
    private static final String STYLE_UNDERLINE = "u";

    private final List<SubtitleIndex> subtitles = new ArrayList<>();
    private final File file;
    private final Charset encoding;

    private Stage stage = Stage.INDEX;
    private SubtitleIndex.SubtitleIndexBuilder subtitleBuilder;
    private List<SubtitleLine> lines;

    private SrtParser(File file, Charset encoding) {
        this.file = file;
        this.encoding = encoding;
    }

    public static List<SubtitleIndex> parse(File file, Charset encoding) {
        Assert.notNull(file, "file cannot be null");
        SrtParser parser = new SrtParser(file, encoding);

        try {
            return parser.read();
        } catch (IOException ex) {
            throw new SubtitleParsingException("Failed to parse file, " + ex.getMessage(), ex);
        }
    }

    private List<SubtitleIndex> read() throws IOException {
        // read the subtitle file to a string an remove the empty lines at the end
        List<String> lines = FileUtils.readLines(file, encoding);

        for (int lineIndex = 0; lineIndex < lines.size(); lineIndex++) {
            String line = lines.get(lineIndex);

            // check if we've reached the end of the current subtitle
            if (StringUtils.isBlank(line))
                stage = Stage.FINISH;

            switch (stage) {
                case INDEX:
                    createNewSubtitle();
                    readIndex(lineIndex, line);
                    break;
                case TIME:
                    readTime(lineIndex, line);
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
        subtitleBuilder = SubtitleIndex.builder();
    }

    private void readIndex(int lineIndex, String line) {
        // remove hidden characters with regex
        Matcher matcher = INDEX_PATTERN.matcher(line);

        // check if any index number can be found in the line
        if (matcher.find()) {
            try {
                subtitleBuilder.index(Long.parseLong(matcher.group(0)));
                nextStage();
            } catch (NumberFormatException ex) {
                String message = MessageFormat.format("Failed to parse subtitle index at line {0}, {1}", lineIndex, ex.getMessage());
                throw new SubtitleParsingException(message, ex);
            }
        } else {
            String message = MessageFormat.format("Failed to read subtitle index at line {0}, \"{1}\" has no index number", lineIndex, line);
            throw new SubtitleParsingException(message);
        }
    }

    private void readTime(int lineIndex, String line) {
        Matcher matcher = TIME_PATTERN.matcher(line);

        if (matcher.matches()) {
            subtitleBuilder
                    .startTime(toMillis(LocalTime.parse(matcher.group(1).replace(",", "."), TIME_FORMAT)))
                    .endTime(toMillis(LocalTime.parse(matcher.group(2).replace(",", "."), TIME_FORMAT)));

            nextStage();
        } else {
            String message = MessageFormat.format("Failed to parse subtitle time at line {0}, invalid time line format \"{1}\"", lineIndex, line);
            throw new SubtitleParsingException(message);
        }
    }

    private void readText(String line) {
        if (lines == null)
            lines = new ArrayList<>();

        var matcher = TEXT_PATTERN.matcher(line);
        var subtitleTexts = new ArrayList<SubtitleText>();

        while (matcher.find()) {
            var text = matcher.group(3);
            var style = getStyle(matcher);

            subtitleTexts.add(SubtitleText.builder()
                    .text(text)
                    .italic(style.equals(STYLE_ITALIC))
                    .bold(style.equals(STYLE_BOLD))
                    .underline(style.equals(STYLE_UNDERLINE))
                    .build());
        }

        lines.add(new SubtitleLine(subtitleTexts));
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

    private String getStyle(Matcher matcher) {
        final var firstStylePosition = matcher.group(2);
        final var lastStylePosition = matcher.group(5);

        return Optional.ofNullable(firstStylePosition)
                .orElse(Optional.ofNullable(lastStylePosition)
                        .orElse(StringUtils.EMPTY));
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
