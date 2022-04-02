package com.github.yoep.provider.anime.parsers.imdb;

import lombok.AccessLevel;
import lombok.NoArgsConstructor;

import java.util.Objects;
import java.util.Optional;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

@NoArgsConstructor(access = AccessLevel.PRIVATE)
public class IdParser {
    private static final Pattern ID_PATTERN = Pattern.compile("(tt[0-9]+)");

    public static Optional<String> extractId(String rawHrefLink) {
        Objects.requireNonNull(rawHrefLink, "rawHrefLink cannot be null");
        var matcher = ID_PATTERN.matcher(rawHrefLink);

        return Optional.of(matcher)
                .filter(Matcher::find)
                .map(e -> e.group(0));
    }
}
