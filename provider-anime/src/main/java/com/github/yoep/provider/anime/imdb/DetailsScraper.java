package com.github.yoep.provider.anime.imdb;

import com.github.yoep.popcorn.backend.config.properties.PopcornProperties;
import com.github.yoep.popcorn.backend.media.providers.MediaRetrievalException;
import com.github.yoep.popcorn.backend.media.providers.models.Images;
import com.github.yoep.provider.anime.media.models.Anime;
import com.github.yoep.provider.anime.parsers.imdb.RuntimeParser;
import com.github.yoep.provider.anime.parsers.imdb.YearParser;
import lombok.extern.slf4j.Slf4j;
import org.jsoup.Jsoup;
import org.jsoup.nodes.Document;
import org.jsoup.nodes.Element;
import org.springframework.stereotype.Service;
import org.springframework.web.reactive.function.client.WebClient;
import org.springframework.web.util.UriComponentsBuilder;

import java.net.URI;
import java.time.Duration;
import java.util.Collections;
import java.util.List;
import java.util.Objects;
import java.util.Optional;
import java.util.stream.Collectors;

@Slf4j
@Service
public record DetailsScraper(WebClient webClient,
                             PopcornProperties popcornConfig) {
    static final String TITLE_TAG = "h1";
    static final String METADATA_CLASS = "ipc-inline-list";
    static final String IMAGE_POSTER_CLASS = "ipc-poster";
    static final String IMAGE_FANART_CLASS = "ipc-slate";
    static final String PLOT_ATTR_KEY = "data-testid";
    static final String PLOT_ATTR_VALUE = "plot";
    static final String GENRES_ATTR_KEY = "data-testid";
    static final String GENRES_ATTR_VALUE = "genres";
    static final String IMAGE_TAG = "img";
    static final String IMAGE_ATTRIBUTE = "src";

    public Anime retrieveDetails(String imdbId) {
        Objects.requireNonNull(imdbId, "imdbId cannot be null");
        var uri = buildRequestUri(imdbId);

        var response = webClient.get()
                .uri(uri)
                .retrieve()
                .bodyToMono(String.class)
                .block(Duration.ofSeconds(10));

        if (response != null) {
            var document = Jsoup.parse(response);

            return mapItemToModel(imdbId, document);
        } else {
            throw new MediaRetrievalException(uri, "Empty body has been returned");
        }
    }

    private URI buildRequestUri(String imdbId) {
        return UriComponentsBuilder.fromUri(popcornConfig.getImdb().getUrl())
                .path("/title/{id}")
                .build(imdbId);
    }

    private Anime mapItemToModel(String imdbId, Document document) {
        return Anime.builder()
                .imdbId(imdbId)
                .nyaaId(imdbId)
                .title(extractTitle(document))
                .year(extractYear(document))
                .runtime(extractRuntime(document))
                .images(extractImages(document))
                .synopsis(extractSynopsis(document))
                .genres(extractGenres(document))
                .episodes(Collections.emptyList())
                .build();
    }

    private static String extractTitle(Document document) {
        return Optional.ofNullable(document.getElementsByTag(TITLE_TAG).first())
                .map(Element::text)
                .orElse("Unknown");
    }

    private static String extractYear(Document document) {
        return Optional.ofNullable(document.getElementsByClass(METADATA_CLASS).first())
                .map(e -> e.getAllElements().get(2))
                .map(Element::text)
                .map(YearParser::extractStartYearFromDetails)
                .orElse(null);
    }

    private static Integer extractRuntime(Document document) {
        return Optional.ofNullable(document.getElementsByClass(METADATA_CLASS).first())
                .map(e -> e.getAllElements().get(4))
                .map(Element::text)
                .map(RuntimeParser::extractRuntime)
                .orElse(null);
    }

    private static String extractSynopsis(Document document) {
        return Optional.ofNullable(document.getElementsByAttributeValue(PLOT_ATTR_KEY, PLOT_ATTR_VALUE).first())
                .map(e -> e.child(0))
                .map(Element::text)
                .orElse(null);
    }

    private static List<String> extractGenres(Document document) {
        return Optional.ofNullable(document.getElementsByAttributeValue(GENRES_ATTR_KEY, GENRES_ATTR_VALUE).first())
                .map(Element::children)
                .map(e -> e.stream()
                        .map(Element::text)
                        .collect(Collectors.toList()))
                .orElse(Collections.emptyList());
    }

    private static Images extractImages(Document document) {
        var poster = Optional.ofNullable(document.getElementsByClass(IMAGE_POSTER_CLASS).first())
                .map(e -> e.getElementsByTag(IMAGE_TAG))
                .map(e -> e.attr(IMAGE_ATTRIBUTE));
        var fanart = Optional.ofNullable(document.getElementsByClass(IMAGE_FANART_CLASS).first())
                .map(e -> e.getElementsByTag(IMAGE_TAG))
                .map(e -> e.attr(IMAGE_ATTRIBUTE));

        return Images.builder()
                .poster(poster.orElse(null))
                .fanart(fanart.orElse(null))
                .banner(fanart.orElse(null))
                .build();
    }
}
