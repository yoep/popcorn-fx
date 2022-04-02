package com.github.yoep.provider.anime.parsers.imdb;

import lombok.AccessLevel;
import lombok.NoArgsConstructor;

import java.util.Optional;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

@NoArgsConstructor(access = AccessLevel.PRIVATE)
public class YearParser {
    private static final Pattern SEARCH_START_YEAR_PATTERN = Pattern.compile("\\(([0-9]+)");
    private static final Pattern DETAILS_START_YEAR_PATTERN = Pattern.compile("([0-9]{4})-");

    public static String extractStartYearFromSearch(String rawYearValue) {
        return Optional.ofNullable(rawYearValue)
                .map(SEARCH_START_YEAR_PATTERN::matcher)
                .filter(Matcher::find)
                .map(e -> e.group(1))
                .orElse(null);
    }

    public static String extractStartYearFromDetails(String rawYearValue) {
        return Optional.ofNullable(rawYearValue)
                .map(DETAILS_START_YEAR_PATTERN::matcher)
                .filter(Matcher::find)
                .map(e -> e.group(1))
                .orElse(null);
    }
}
