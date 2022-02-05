package com.github.yoep.provider.anime.parsers;

import lombok.AccessLevel;
import lombok.NoArgsConstructor;

import java.util.Objects;
import java.util.Optional;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

@NoArgsConstructor(access = AccessLevel.PRIVATE)
public class EpisodeParser {
    private static final Pattern EPISODE_PATTERN = Pattern.compile("\\s([0-9]+)\\s");

    public static Optional<Integer> extractEpisode(String filename) {
        Objects.requireNonNull(filename, "filename cannot be null");
        var matcher = EPISODE_PATTERN.matcher(filename);

        return Optional.of(matcher)
                .filter(Matcher::find)
                .map(e -> e.group(1))
                .map(Integer::parseInt);
    }
}
