package com.github.yoep.popcorn.backend.subtitles.parser;

import com.github.yoep.popcorn.backend.subtitles.model.SubtitleLine;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleText;
import lombok.AccessLevel;
import lombok.NoArgsConstructor;
import org.apache.commons.lang3.StringUtils;

import javax.validation.constraints.NotNull;
import java.text.MessageFormat;
import java.util.ArrayList;
import java.util.Objects;
import java.util.Optional;
import java.util.regex.Matcher;
import java.util.regex.Pattern;
import java.util.stream.Collectors;

/**
 * The style parser parses text from and to subtitle files based on the basic styles.
 * Complex styles, e.g. colors, are ignored/removed from the line.
 * <p>
 * The following styles are supported:
 * <ul>
 *     <li>Italic - i</li>
 *     <li>Bold - b</li>
 *     <li>Underline - u</li>
 * </ul>
 */
@NoArgsConstructor(access = AccessLevel.PRIVATE)
public class StyleParser {
    static final Pattern TEXT_PATTERN = Pattern.compile("(<([^>]*)>)?([^<]+)(</([^>]*)>)?");

    static final String STYLE_ITALIC = "i";
    static final String STYLE_BOLD = "b";
    static final String STYLE_UNDERLINE = "u";

    /**
     * Parse the given text line which can contain styles to a {@link SubtitleLine}.
     *
     * @param textLine The subtitle text line to parse.
     * @return Returns the parsed text line with styles if applicable.
     */
    public static SubtitleLine parseLineStyle(@NotNull String textLine) {
        Objects.requireNonNull(textLine, "textLine cannot be null");
        var matcher = TEXT_PATTERN.matcher(textLine);
        var subtitleTexts = new ArrayList<SubtitleText>();

        while (matcher.find()) {
            var text = matcher.group(3);
            var style = retrieveStyleIndicator(matcher);

            subtitleTexts.add(SubtitleText.builder()
                    .text(text)
                    .italic(style.equalsIgnoreCase(STYLE_ITALIC))
                    .bold(style.equalsIgnoreCase(STYLE_BOLD))
                    .underline(style.equalsIgnoreCase(STYLE_UNDERLINE))
                    .build());
        }

        return new SubtitleLine(subtitleTexts);
    }

    /**
     * Write the given subtitle line back to the original subtitle file format.
     *
     * @param subtitleLine The subtitle to write.
     * @return Returns the original subtitle format.
     */
    public static String write(@NotNull SubtitleLine subtitleLine) {
        Objects.requireNonNull(subtitleLine, "subtitleLine cannot be null");
        return subtitleLine.texts().stream()
                .map(StyleParser::write)
                .collect(Collectors.joining(""));
    }

    private static String write(SubtitleText subtitleText) {
        var text = subtitleText.text();

        if (containsStyleInformation(subtitleText)) {
            var tag = "";

            if (subtitleText.italic()) {
                tag = STYLE_ITALIC;
            } else if (subtitleText.bold()) {
                tag = STYLE_BOLD;
            } else if (subtitleText.underline()) {
                tag = STYLE_UNDERLINE;
            }

            text = MessageFormat.format("<{0}>{1}</{0}>", tag, text);
        }

        return text;
    }

    private static boolean containsStyleInformation(SubtitleText subtitleText) {
        return subtitleText.italic() || subtitleText.bold() || subtitleText.underline();
    }

    private static String retrieveStyleIndicator(Matcher matcher) {
        final var firstStylePosition = matcher.group(2);
        final var lastStylePosition = matcher.group(5);

        return Optional.ofNullable(firstStylePosition)
                .orElse(Optional.ofNullable(lastStylePosition)
                        .orElse(StringUtils.EMPTY));
    }
}
