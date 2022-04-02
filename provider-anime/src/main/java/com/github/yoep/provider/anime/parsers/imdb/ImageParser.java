package com.github.yoep.provider.anime.parsers.imdb;

import lombok.AccessLevel;
import lombok.NoArgsConstructor;

import java.util.Optional;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

@NoArgsConstructor(access = AccessLevel.PRIVATE)
public class ImageParser {
    private static final Pattern PATTERN = Pattern.compile("M\\/([a-zA-Z0-9]+)@\\._V1_UX67_CR0,0,67,98_AL_\\.jpg");

    public static String extractImage(String src) {
        return Optional.ofNullable(src)
                .map(PATTERN::matcher)
                .filter(Matcher::find)
                .map(e -> e.group(1))
                .orElse(null);
    }
}
