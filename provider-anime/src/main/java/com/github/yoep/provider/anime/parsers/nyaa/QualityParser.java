package com.github.yoep.provider.anime.parsers.nyaa;

import lombok.AccessLevel;
import lombok.NoArgsConstructor;

import java.util.Optional;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

@NoArgsConstructor(access = AccessLevel.PRIVATE)
public class QualityParser {
    private static final Pattern PIXEL_PATTERN = Pattern.compile("([0-9]+)p", Pattern.CASE_INSENSITIVE);
    private static final Pattern RESOLUTION_PATTERN = Pattern.compile("[0-9]+x([0-9]+)", Pattern.CASE_INSENSITIVE);
    private static final String QUALITY_IDENTIFIER = "p";

    public static Optional<String> extractQuality(String rawTitle) {
        return Optional.ofNullable(rawTitle)
                .flatMap(QualityParser::extractPixelQuality)
                .or(() -> extractResolutionQuality(rawTitle))
                .map(QualityParser::addQualityIdentifier);
    }

    private static Optional<String> extractPixelQuality(String rawTitle) {
        return Optional.of(PIXEL_PATTERN.matcher(rawTitle))
                .filter(Matcher::find)
                .map(e -> e.group(1));
    }

    private static Optional<String> extractResolutionQuality(String rawTitle) {
        return Optional.of(RESOLUTION_PATTERN.matcher(rawTitle))
                .filter(Matcher::find)
                .map(e -> e.group(1));
    }

    private static String addQualityIdentifier(String quality) {
        return quality + QUALITY_IDENTIFIER;
    }
}
