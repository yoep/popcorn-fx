package com.github.yoep.provider.anime.parsers.imdb;

import lombok.AccessLevel;
import lombok.NoArgsConstructor;

import java.util.Objects;
import java.util.Optional;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

@NoArgsConstructor(access = AccessLevel.PRIVATE)
public class RuntimeParser {
    private static final Pattern RUNTIME_PATTERN = Pattern.compile("[0-9]+");

    public static Integer extractRuntime(String rawValue) {
        Objects.requireNonNull(rawValue, "rawValue cannot be null");
        var matcher = RUNTIME_PATTERN.matcher(rawValue);

        return Optional.of(matcher)
                .filter(Matcher::find)
                .map(e -> e.group(0))
                .map(Integer::parseInt)
                .orElse(null);
    }
}
