package com.github.yoep.provider.anime.parsers;

import lombok.AccessLevel;
import lombok.NoArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.lang3.StringUtils;

import javax.validation.constraints.NotNull;
import java.time.Instant;
import java.time.ZoneId;
import java.time.ZonedDateTime;
import java.util.Objects;
import java.util.Optional;

@Slf4j
@NoArgsConstructor(access = AccessLevel.PRIVATE)
public class DateParser {
    public static String convertDateToYear(@NotNull String htmlDateValue) {
        Objects.requireNonNull(htmlDateValue, "htmlDateValue cannot be null");
        return Optional.of(htmlDateValue)
                .filter(StringUtils::isNotEmpty)
                .flatMap(DateParser::tryParsingLongValue)
                .map(Instant::ofEpochSecond)
                .map(DateParser::toSystemZone)
                .map(ZonedDateTime::getYear)
                .map(String::valueOf)
                .orElse(null);
    }

    private static ZonedDateTime toSystemZone(Instant epochInstant) {
        return epochInstant.atZone(ZoneId.systemDefault());
    }

    private static Optional<Long> tryParsingLongValue(String epoch) {
        try {
            return Optional.of(Long.parseLong(epoch));
        } catch (NumberFormatException ex) {
            log.warn("Failed to parse epoch value, {}", ex.getMessage(), ex);
        }

        return Optional.empty();
    }
}
