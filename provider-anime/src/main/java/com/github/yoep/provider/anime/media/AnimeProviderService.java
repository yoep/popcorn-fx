package com.github.yoep.provider.anime.media;

import com.github.yoep.popcorn.backend.config.properties.PopcornProperties;
import com.github.yoep.popcorn.backend.media.filters.models.Category;
import com.github.yoep.popcorn.backend.media.filters.models.Genre;
import com.github.yoep.popcorn.backend.media.filters.models.SortBy;
import com.github.yoep.popcorn.backend.media.providers.AbstractProviderService;
import com.github.yoep.popcorn.backend.media.providers.MediaException;
import com.github.yoep.popcorn.backend.media.providers.models.Episode;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.providers.models.MediaTorrentInfo;
import com.github.yoep.popcorn.backend.settings.SettingsService;
import com.github.yoep.provider.anime.media.mappers.AnimeMapper;
import com.github.yoep.provider.anime.media.models.Anime;
import com.github.yoep.provider.anime.parsers.*;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.lang3.StringUtils;
import org.jsoup.Jsoup;
import org.jsoup.nodes.Element;
import org.jsoup.select.Elements;
import org.springframework.data.domain.Page;
import org.springframework.data.domain.PageImpl;
import org.springframework.stereotype.Service;
import org.springframework.web.client.RestTemplate;
import org.springframework.web.util.UriComponentsBuilder;

import java.net.URI;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.concurrent.CompletableFuture;
import java.util.stream.Collectors;

@Slf4j
@Service
public class AnimeProviderService extends AbstractProviderService<Anime> {
    private static final Category CATEGORY = Category.ANIME;
    private static final String QUERY_FILTER_KEY = "f";
    private static final String QUERY_GENRE_KEY = "c";
    private static final String QUERY_SEARCH_KEY = "q";
    private static final String QUERY_PAGE_KEY = "p";
    private static final String DATE_ATTRIBUTE = "data-timestamp";
    private static final String SCRAPER_FIELD_TITLE = "td:nth-child(2) a:last-child";
    private static final String SCRAPER_FIELD_DATE = "a[" + DATE_ATTRIBUTE + "]";
    private static final String DEFAULT_QUALITY = "480p";

    public AnimeProviderService(RestTemplate restTemplate,
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
        return CompletableFuture.completedFuture(getDetailsInternal(imdbId));
    }

    @Override
    public CompletableFuture<Media> retrieveDetails(final Media media) {
        var anime = getDetailsInternal(media.getId());
        Media result;

        // based on the available files
        // we'll return a Movie or Show of the anime information
        if (anime.getEpisodes().size() == 1) {
            result = AnimeMapper.toMovie(anime);
        } else {
            result = AnimeMapper.toShow(anime);
        }

        return CompletableFuture.completedFuture(result);
    }

    public Page<Anime> getPage(Genre genre, SortBy sortBy, String keywords, int page) {
        return invokeWithUriProvider(apiUri -> {
            var uri = buildSearchRequestUri(apiUri, genre, sortBy, keywords, page);
            var result = new ArrayList<Anime>();

            log.debug("Retrieving anime provider page \"{}\"", uri);
            var response = restTemplate.getForEntity(uri, String.class);

            if (response.hasBody()) {
                var document = Jsoup.parse(response.getBody());
                var torrents = document.select(".torrent-list tbody tr");

                for (Element torrent : torrents) {
                    var title = torrent.select(SCRAPER_FIELD_TITLE).text();
                    var id = IdParser.extractId(torrent.select(SCRAPER_FIELD_TITLE).attr("href"));

                    result.add(Anime.builder()
                            .nyaaId(id)
                            .imdbId(id)
                            .title(TitleParser.normaliseTitle(title))
                            .year(DateParser.convertDateToYear(torrent.select(SCRAPER_FIELD_DATE).attr(DATE_ATTRIBUTE)))
                            .build());
                }
            }

            return new PageImpl<>(result);
        });
    }

    private Anime getDetailsInternal(String imdbId) {
        return invokeWithUriProvider(apiUri -> {
            var uri = buildDetailsRequestUri(apiUri, imdbId);

            log.debug("Retrieving anime provider details of \"{}\"", uri);
            var response = restTemplate.getForEntity(uri, String.class);

            if (response.hasBody()) {
                var document = Jsoup.parse(response.getBody());
                var title = document.select(".panel-title").text();
                var year = DateParser.convertDateToYear(document.select(SCRAPER_FIELD_DATE).attr(DATE_ATTRIBUTE));
                var torrentUrl = document.select(".panel-footer a:first-child").attr("href");
                var files = document.select(".torrent-file-list li:has(.fa-file)");
                var episodes = extractEpisodesFromFiles(apiUri, torrentUrl, files);

                return Anime.builder()
                        .nyaaId(imdbId)
                        .imdbId(imdbId)
                        .title(TitleParser.normaliseTitle(title))
                        .year(year)
                        .episodes(episodes)
                        .build();
            } else {
                throw new MediaException("No details response available for " + uri);
            }
        });
    }

    private List<Episode> extractEpisodesFromFiles(URI apiUri, String torrentUrl, Elements files) {
        return files.stream()
                .map(Element::text)
                .map(e -> createEpisode(apiUri, torrentUrl, e))
                .collect(Collectors.toList());
    }

    private Episode createEpisode(URI apiUri, String torrentUrl, String filename) {
        var quality = QualityParser.extractQuality(filename)
                .orElse(DEFAULT_QUALITY);

        return Episode.builder()
                .title(TitleParser.normaliseTitle(filename))
                .season(1)
                .episode(EpisodeParser.extractEpisode(filename))
                .torrents(Collections.singletonMap(quality, MediaTorrentInfo.builder()
                        .file(filename)
                        .url(apiUri + torrentUrl)
                        .build()))
                .build();
    }

    private static URI buildDetailsRequestUri(URI apiUri, String id) {
        return UriComponentsBuilder.fromUri(apiUri)
                .path("/view/{id}")
                .build(id);
    }

    private static URI buildSearchRequestUri(URI apiUri, Genre genre, SortBy sortBy, String keywords, int page) {
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
