package com.github.yoep.provider.anime.imdb.parsers;

import lombok.AccessLevel;
import lombok.NoArgsConstructor;

import java.util.Objects;
import java.util.Optional;
import java.util.regex.Pattern;

@NoArgsConstructor(access = AccessLevel.PRIVATE)
public class IdParser {
    private static final Pattern ID_PATTERN = Pattern.compile("(tt[0-9]+)");

    public static Optional<String> extractId(String rawHrefLink) {
        Objects.requireNonNull(rawHrefLink, "rawHrefLink cannot be null");
        var matcher = ID_PATTERN.matcher(rawHrefLink);

        if (matcher.find()) {
            return Optional.of(matcher.group(0));
        }

        return Optional.empty();
    }
}
