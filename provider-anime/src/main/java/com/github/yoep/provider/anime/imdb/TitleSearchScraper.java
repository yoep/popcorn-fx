package com.github.yoep.provider.anime.imdb;

import com.github.yoep.popcorn.backend.config.properties.PopcornProperties;
import com.github.yoep.popcorn.backend.media.filters.models.Genre;
import com.github.yoep.popcorn.backend.media.filters.models.SortBy;
import com.github.yoep.popcorn.backend.media.providers.MediaRetrievalException;
import com.github.yoep.popcorn.backend.media.providers.models.Images;
import com.github.yoep.provider.anime.media.models.Anime;
import com.github.yoep.provider.anime.parsers.imdb.*;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.lang3.StringUtils;
import org.jsoup.Jsoup;
import org.jsoup.nodes.Element;
import org.jsoup.select.Elements;
import org.springframework.data.domain.Page;
import org.springframework.data.domain.PageImpl;
import org.springframework.data.domain.PageRequest;
import org.springframework.stereotype.Service;
import org.springframework.web.reactive.function.client.WebClient;
import org.springframework.web.util.UriComponentsBuilder;

import java.net.URI;
import java.text.MessageFormat;
import java.time.Duration;
import java.util.Optional;

/**
 * Scraper service which uses the title search from IMDB.
 */
@Slf4j
@Service
public record TitleSearchScraper(WebClient webClient,
                                 PopcornProperties popcornConfig) {
    static final String QUERY_GENRE_KEY = "genres";
    static final String QUERY_GENRE_VALUE = "animation";
    static final String QUERY_EXPLORE_KEY = "explore";
    static final String QUERY_EXPLORE_VALUE = "title_type,genres";
    static final String QUERY_TITLE_TYPE_KEY = "title_type";
    static final String QUERY_TITLE_TYPE_VALUE = "tvSeries";
    static final String QUERY_SORT_KEY = "sort";
    static final String QUERY_PAGE_KEY = "start";
    static final String QUERY_TITLE_KEY = "title";
    static final String RESULT_ITEM_CLASS = "lister-item";
    static final String TITLE_CLASS = "lister-item-header";
    static final String IMAGE_CLASS = "lister-item-image";
    static final String YEAR_CLASS = "lister-item-year";
    static final String RUNTIME_CLASS = "runtime";
    static final String LINK_TAG = "a";
    static final String LINK_ATTRIBUTE = "href";
    static final String IMAGE_TAG = "img";
    static final String IMAGE_ATTRIBUTE = "loadlate";
    static final String IMAGE_LINK = "https://m.media-amazon.com/images/M/{0}@._V1_QL75_UY281_CR1,0,190,281_.jpg";

    public Page<Anime> retrievePage(Genre genre, SortBy sortBy, int page, String keywords) {
        var uri = buildRequestUri(sortBy, page, keywords);

        var response = webClient.get()
                .uri(uri)
                .retrieve()
                .bodyToMono(String.class)
                .block(Duration.ofSeconds(30));

        if (response != null) {
            var document = Jsoup.parse(response);
            var items = document.getElementsByClass(RESULT_ITEM_CLASS);

            var animes = items.stream()
                    .map(this::mapItemToModel)
                    .toList();

            return new PageImpl<>(animes, PageRequest.of(page, 50), Long.MAX_VALUE);
        } else {
            throw new MediaRetrievalException(uri, "Empty body has been returned");
        }
    }

    private URI buildRequestUri(SortBy sortBy, int page, String keywords) {
        var uriBuilder = UriComponentsBuilder.fromUri(popcornConfig.getImdb().getUrl())
                .path("search/title")
                .queryParam(QUERY_GENRE_KEY, QUERY_GENRE_VALUE)
                .queryParam(QUERY_EXPLORE_KEY, QUERY_EXPLORE_VALUE)
                .queryParam(QUERY_TITLE_TYPE_KEY, QUERY_TITLE_TYPE_VALUE)
                .queryParam(QUERY_PAGE_KEY, 50 * (page - 1))
                .queryParam(QUERY_SORT_KEY, mapSortByToImdbValue(sortBy));

        if (StringUtils.isNotEmpty(keywords)) {
            uriBuilder.queryParam(QUERY_TITLE_KEY, keywords);
        }

        return uriBuilder
                .build()
                .toUri();
    }

    private Anime mapItemToModel(Element item) {
        var id = extractId(item);

        return Anime.builder()
                .nyaaId(id)
                .imdbId(id)
                .title(extractTitle(item))
                .images(extractImages(item))
                .year(extractYear(item))
                .rating(RatingParser.extractRating(item))
                .runtime(extractRuntime(item))
                .build();
    }

    private static String extractId(Element item) {
        return Optional.ofNullable(item.getElementsByClass(TITLE_CLASS).first())
                .map(e -> e.getElementsByTag(LINK_TAG))
                .map(e -> e.attr(LINK_ATTRIBUTE))
                .flatMap(IdParser::extractId)
                .orElse("Unknown");
    }

    private static String extractTitle(Element item) {
        return Optional.ofNullable(item.getElementsByClass(TITLE_CLASS).first())
                .map(e -> e.getElementsByTag(LINK_TAG))
                .map(Elements::text)
                .orElse("Unknown");
    }

    private static Images extractImages(Element item) {
        return Optional.ofNullable(item.getElementsByClass(IMAGE_CLASS).first())
                .map(e -> e.getElementsByTag(IMAGE_TAG))
                .map(e -> e.attr(IMAGE_ATTRIBUTE))
                .map(ImageParser::extractImage)
                .map(e -> MessageFormat.format(IMAGE_LINK, e))
                .map(e -> Images.builder()
                        .fanart(e)
                        .poster(e)
                        .build())
                .orElse(Images.builder().build());
    }

    private static String extractYear(Element item) {
        return Optional.ofNullable(item.getElementsByClass(YEAR_CLASS).first())
                .map(Element::text)
                .map(YearParser::extractStartYearFromSearch)
                .orElse("");
    }

    private static Integer extractRuntime(Element item) {
        return Optional.ofNullable(item.getElementsByClass(RUNTIME_CLASS).first())
                .map(Element::text)
                .map(RuntimeParser::extractRuntime)
                .orElse(null);
    }

    private static String mapSortByToImdbValue(SortBy sortBy) {
        return switch (sortBy.getKey()) {
            case "name" -> "alpha,asc";
            case "rating" -> "user_rating,asc";
            case "year" -> "year,asc";
            default -> "moviemeter,asc";
        };
    }
}
