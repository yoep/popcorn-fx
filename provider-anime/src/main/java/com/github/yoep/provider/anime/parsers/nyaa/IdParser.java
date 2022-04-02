package com.github.yoep.provider.anime.parsers.nyaa;

import lombok.AccessLevel;
import lombok.NoArgsConstructor;
import org.apache.commons.io.FilenameUtils;

import javax.validation.constraints.NotNull;
import java.util.Objects;
import java.util.Optional;

@NoArgsConstructor(access = AccessLevel.PRIVATE)
public class IdParser {
    public static String extractId(@NotNull String rawHtmlHrefLink) {
        Objects.requireNonNull(rawHtmlHrefLink, "rawHtmlHrefLink cannot be null");
        return Optional.of(rawHtmlHrefLink)
                .map(FilenameUtils::getBaseName)
                .orElse(null);
    }
}
