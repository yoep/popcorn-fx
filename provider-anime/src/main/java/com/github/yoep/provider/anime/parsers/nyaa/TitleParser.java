package com.github.yoep.provider.anime.parsers.nyaa;

import lombok.AccessLevel;
import lombok.NoArgsConstructor;
import org.apache.commons.io.FilenameUtils;

import javax.validation.constraints.NotNull;
import java.util.Objects;
import java.util.Optional;

@NoArgsConstructor(access = AccessLevel.PRIVATE)
public class TitleParser {
    /**
     * Normalise the given title.
     * This method should remove most noise of the title such as quality, year, extension, etc.
     *
     * @param rawTitle The raw title to normalise.
     * @return Returns the normalised title.
     */
    public static String normaliseTitle(@NotNull String rawTitle) {
        Objects.requireNonNull(rawTitle, "rawTitle cannot be null");
        return Optional.of(rawTitle)
                .map(TitleParser::removeTags)
                .map(TitleParser::removeAdditionalInfo)
                .map(TitleParser::removeFileSize)
                .map(TitleParser::removeUnderscore)
                .map(TitleParser::removeDoubleSpaces)
                .map(FilenameUtils::getBaseName)
                .map(String::trim)
                .orElse(null);
    }

    private static String removeTags(String rawTitle) {
        return rawTitle.replaceAll("\\[[a-zA-Z0-9\\-!_\\s]*\\]", "");
    }

    private static String removeAdditionalInfo(String rawTitle) {
        return rawTitle.replaceAll("\\([0-9a-zA-Z\\s]+\\)", "");
    }

    private static String removeFileSize(String rawTitle) {
        return rawTitle.replaceAll("\\(?[0-9\\.]+\\s(MiB|GiB)\\)?", "");
    }

    private static String removeUnderscore(String rawTitle) {
        return rawTitle.replaceAll("_", " ");
    }

    private static String removeDoubleSpaces(String rawTitle) {
        return rawTitle.replaceAll("\\s\\s", " ");
    }
}
