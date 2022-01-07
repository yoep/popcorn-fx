package com.github.yoep.provider.anime.parsers;

import lombok.NoArgsConstructor;
import org.apache.commons.io.FilenameUtils;

import javax.validation.constraints.NotNull;
import java.util.Objects;
import java.util.Optional;

@NoArgsConstructor
public class TitleParser {
    public static String cleanTitle(@NotNull String rawTitle) {
        Objects.requireNonNull(rawTitle, "rawTitle cannot be null");
        return Optional.of(rawTitle)
                .map(FilenameUtils::getBaseName)
                .map(e -> e.replaceAll("\\[[a-zA-Z0-9\\-!_\\s]*\\]", ""))
                .map(String::trim)
                .orElse(null);
    }
}
