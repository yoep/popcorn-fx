package com.github.yoep.provider.anime.imdb;

import com.github.yoep.popcorn.backend.config.properties.PopcornProperties;
import com.github.yoep.popcorn.backend.media.filters.models.Genre;
import com.github.yoep.popcorn.backend.media.filters.models.SortBy;
import com.github.yoep.popcorn.backend.media.providers.MediaRetrievalException;
import com.github.yoep.popcorn.backend.media.providers.models.Images;
import com.github.yoep.provider.anime.media.models.Anime;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.jsoup.Jsoup;
import org.jsoup.nodes.Element;
import org.jsoup.select.Elements;
import org.springframework.data.domain.Page;
import org.springframework.data.domain.PageImpl;
import org.springframework.data.domain.PageRequest;
import org.springframework.stereotype.Service;
import org.springframework.web.reactive.function.client.WebClient;
import org.springframework.web.util.UriComponentsBuilder;

import java.time.Duration;
import java.util.Optional;

@Slf4j
@Service
@RequiredArgsConstructor
public class ImdbScraperService {
    static final String QUERY_GENRE_KEY = "genres";
    static final String QUERY_GENRE_VALUE = "animation";
    static final String QUERY_EXPLORE_KEY = "explore";
    static final String QUERY_EXPLORE_VALUE = "title_type,genres";
    static final String QUERY_TITLE_TYPE_KEY = "title_type";
    static final String QUERY_TITLE_TYPE_VALUE = "tvSeries";
    static final String QUERY_SORT_KEY = "sort";
    static final String RESULT_ITEM_CLASS = "lister-item";
    static final String TITLE_CLASS = "lister-item-header";
    static final String IMAGE_CLASS = "lister-item-image";
    static final String LINK_TAG = "a";
    static final String LINK_ATTRIBUTE = "href";

    private final WebClient webClient;
    private final PopcornProperties popcornConfig;

    public Page<Anime> retrievePage(Genre genre, SortBy sortBy, int page, String keywords) {
        var uri = UriComponentsBuilder.fromUri(popcornConfig.getImdb().getUrl())
                .path("search/title")
                .queryParam(QUERY_GENRE_KEY, QUERY_GENRE_VALUE)
                .queryParam(QUERY_EXPLORE_KEY, QUERY_EXPLORE_VALUE)
                .queryParam(QUERY_TITLE_TYPE_KEY, QUERY_TITLE_TYPE_VALUE)
                .build()
                .toUri();

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

    private Anime mapItemToModel(Element item) {
        var id = extractId(item);
        var title = extractTitle(item);

        return Anime.builder()
                .nyaaId(id)
                .imdbId(id)
                .title(title)
                .images(Images.builder().build())
                .build();
    }

    private static String extractId(Element item) {
        return Optional.ofNullable(item.getElementsByClass(TITLE_CLASS).first())
                .map(e -> e.getElementsByTag(LINK_TAG))
                .map(e -> e.attr(LINK_ATTRIBUTE))
                .orElse("Unknown");
    }

    private static String extractTitle(Element item) {
        return Optional.ofNullable(item.getElementsByClass(TITLE_CLASS).first())
                .map(e -> e.getElementsByTag(LINK_TAG))
                .map(Elements::text)
                .orElse("Unknown");
    }
}
