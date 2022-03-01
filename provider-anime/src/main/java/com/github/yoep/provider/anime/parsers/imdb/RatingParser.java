package com.github.yoep.provider.anime.parsers.imdb;

import com.github.yoep.popcorn.backend.media.providers.models.Rating;
import lombok.AccessLevel;
import lombok.NoArgsConstructor;
import org.jsoup.nodes.Element;

import java.util.Objects;
import java.util.Optional;

@NoArgsConstructor(access = AccessLevel.PRIVATE)
public class RatingParser {
    static final String RATING_CLASS = "ratings-imdb-rating";
    static final String STRONG_TAG = "strong";

    public static Rating extractRating(Element item) {
        Objects.requireNonNull(item, "item cannot be null");
        return Optional.ofNullable(item.getElementsByClass(RATING_CLASS).first())
                .map(e -> e.getElementsByTag(STRONG_TAG).first())
                .map(Element::text)
                .map(Double::parseDouble)
                .map(e -> Rating.builder()
                        .percentage((int) (e * 10))
                        .build())
                .orElse(null);
    }
}
