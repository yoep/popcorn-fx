package com.github.yoep.provider.anime.parsers.imdb;

import lombok.AccessLevel;
import lombok.NoArgsConstructor;

import java.util.Objects;
import java.util.Optional;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

@NoArgsConstructor(access = AccessLevel.PRIVATE)
public class YearParser {
    private static final Pattern START_YEAR_PATTERN = Pattern.compile("\\(([0-9]+)");

    public static String extractStartYear(String rawYearValue) {
        Objects.requireNonNull(rawYearValue, "rawYearValue cannot be null");
        var matcher = START_YEAR_PATTERN.matcher(rawYearValue);

        return Optional.of(matcher)
                .filter(Matcher::find)
                .map(e -> e.group(1))
                .orElse(null);
    }
}
