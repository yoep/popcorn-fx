package com.github.yoep.provider.anime.parsers;

import lombok.AccessLevel;
import lombok.NoArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.lang3.StringUtils;

import javax.validation.constraints.NotNull;
import java.time.LocalDateTime;
import java.time.format.DateTimeFormatter;
import java.time.format.DateTimeParseException;
import java.util.Objects;
import java.util.Optional;

@Slf4j
@NoArgsConstructor(access = AccessLevel.PRIVATE)
public class DateParser {
    private static final DateTimeFormatter FORMATTER = DateTimeFormatter.RFC_1123_DATE_TIME;

    public static String convertDateToYear(@NotNull String htmlDateValue) {
        Objects.requireNonNull(htmlDateValue, "htmlDateValue cannot be null");
        return Optional.of(htmlDateValue)
                .filter(StringUtils::isNotEmpty)
                .map(DateParser::tryParsingDate)
                .map(LocalDateTime::getYear)
                .map(String::valueOf)
                .orElse(null);
    }

    private static LocalDateTime tryParsingDate(String value) {
        try {
            return LocalDateTime.parse(value, FORMATTER);
        } catch (DateTimeParseException ex) {
            log.trace("Failed to parse date {}, {}", value, ex.getMessage());
            return null;
        }
    }
}
