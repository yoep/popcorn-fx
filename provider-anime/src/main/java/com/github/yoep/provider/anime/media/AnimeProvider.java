package com.github.yoep.provider.anime.media;

import com.github.yoep.popcorn.backend.config.properties.PopcornProperties;
import com.github.yoep.popcorn.backend.media.filters.models.Category;
import com.github.yoep.popcorn.backend.media.filters.models.Genre;
import com.github.yoep.popcorn.backend.media.filters.models.SortBy;
import com.github.yoep.popcorn.backend.media.providers.AbstractProviderService;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.settings.SettingsService;
import com.github.yoep.provider.anime.media.models.Anime;
import com.github.yoep.provider.anime.parsers.DateParser;
import com.github.yoep.provider.anime.parsers.TitleParser;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.lang3.StringUtils;
import org.jsoup.Jsoup;
import org.jsoup.nodes.Element;
import org.springframework.data.domain.Page;
import org.springframework.data.domain.PageImpl;
import org.springframework.stereotype.Service;
import org.springframework.web.client.RestTemplate;
import org.springframework.web.util.UriComponentsBuilder;

import java.net.URI;
import java.util.ArrayList;
import java.util.Collections;
import java.util.concurrent.CompletableFuture;

@Slf4j
@Service
public class AnimeProvider extends AbstractProviderService<Anime> {
    private static final Category CATEGORY = Category.ANIME;
    private static final String QUERY_FILTER_KEY = "f";
    private static final String QUERY_GENRE_KEY = "c";
    private static final String QUERY_SEARCH_KEY = "q";
    private static final String QUERY_PAGE_KEY = "p";

    public AnimeProvider(RestTemplate restTemplate,
                         PopcornProperties popcornConfig,
                         SettingsService settingsService) {
        super(restTemplate);

        initializeUriProviders(settingsService.getSettings().getServerSettings(), popcornConfig.getProvider(CATEGORY.getProviderName()));
    }

    @Override
    public boolean supports(Category category) {
        return category == CATEGORY;
    }

    @Override
    public CompletableFuture<Page<Anime>> getPage(Genre genre, SortBy sortBy, int page) {
        return CompletableFuture.completedFuture(getPage(genre, sortBy, "", page));
    }

    @Override
    public CompletableFuture<Page<Anime>> getPage(Genre genre, SortBy sortBy, int page, String keywords) {
        return CompletableFuture.completedFuture(getPage(genre, sortBy, keywords, page));
    }

    @Override
    public CompletableFuture<Anime> getDetails(String imdbId) {
        return null;
    }

    @Override
    public CompletableFuture<Media> retrieveDetails(Media media) {
        return null;
    }

    public Page<Anime> getPage(Genre genre, SortBy sortBy, String keywords, int page) {
        return invokeWithUriProvider(apiUri -> {
            var uri = buildRequestUri(apiUri, genre, sortBy, keywords, page);
            var result = new ArrayList<Anime>();

            log.debug("Retrieving anime provider page \"{}\"", uri);
            var response = restTemplate.getForEntity(uri, String.class);

            if (response.hasBody()) {
                var document = Jsoup.parse(response.getBody());
                var torrents = document.select(".torrent-list tbody tr");

                for (Element torrent : torrents) {
                    var title = torrent.select("td:nth-child(2) a:last-child").text();

                    result.add(Anime.builder()
                            .title(TitleParser.cleanTitle(title))
                            .year(DateParser.convertDateToYear(torrent.select("td:nth-child(5)").attr("data-timestamp")))
                            .build());
                }
            }

            return new PageImpl<>(result);
        });
    }

    private static URI buildRequestUri(URI apiUri, Genre genre, SortBy sortBy, String keywords, int page) {
        var uriBuilder = UriComponentsBuilder.fromUri(apiUri)
                .queryParam(QUERY_FILTER_KEY, 2)
                .queryParam(QUERY_GENRE_KEY, genreToQueryValue(genre))
                .queryParam(QUERY_PAGE_KEY, page);

        if (StringUtils.isNotEmpty(keywords)) {
            uriBuilder.queryParam(QUERY_SEARCH_KEY, keywords);
        }

        return uriBuilder.build(Collections.emptyMap());
    }

    private static String genreToQueryValue(Genre genre) {
        switch (genre.getKey()) {
            case "anime-music-video":
                return "1_1";
            case "english-translated":
                return "1_2";
            case "non-english-translated":
                return "1_3";
            case "raw":
                return "1_4";
            default:
                return "1_0";
        }
    }
}
