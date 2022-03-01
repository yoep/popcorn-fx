package com.github.yoep.provider.anime.media;

import com.github.yoep.popcorn.backend.adapters.torrent.TorrentService;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentFileInfo;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentInfo;
import com.github.yoep.popcorn.backend.config.properties.PopcornProperties;
import com.github.yoep.popcorn.backend.media.filters.models.Category;
import com.github.yoep.popcorn.backend.media.filters.models.Genre;
import com.github.yoep.popcorn.backend.media.filters.models.SortBy;
import com.github.yoep.popcorn.backend.media.providers.AbstractProviderService;
import com.github.yoep.popcorn.backend.media.providers.MediaException;
import com.github.yoep.popcorn.backend.media.providers.MediaRetrievalException;
import com.github.yoep.popcorn.backend.media.providers.models.Episode;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.providers.models.MediaTorrentInfo;
import com.github.yoep.popcorn.backend.settings.SettingsService;
import com.github.yoep.provider.anime.imdb.ImdbScraperService;
import com.github.yoep.provider.anime.media.mappers.AnimeMapper;
import com.github.yoep.provider.anime.media.models.Anime;
import com.github.yoep.provider.anime.media.models.Item;
import com.github.yoep.provider.anime.media.models.Nyaa;
import com.github.yoep.provider.anime.parsers.*;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.lang3.StringUtils;
import org.springframework.data.domain.Page;
import org.springframework.stereotype.Service;
import org.springframework.web.client.RestTemplate;
import org.springframework.web.util.UriComponentsBuilder;

import javax.xml.bind.JAXBContext;
import javax.xml.bind.JAXBException;
import java.io.ByteArrayInputStream;
import java.net.URI;
import java.nio.charset.StandardCharsets;
import java.util.Collections;
import java.util.List;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutionException;
import java.util.stream.Collectors;

@Slf4j
@Service
public class AnimeProviderService extends AbstractProviderService<Anime> {
    private static final Category CATEGORY = Category.ANIME;
    private static final String QUERY_FILTER_KEY = "f";
    private static final String QUERY_GENRE_KEY = "c";
    private static final String QUERY_SEARCH_KEY = "q";
    private static final String QUERY_PAGE_KEY = "p";
    private static final String QUERY_PAGE_TYPE_KEY = "page";
    private static final String QUERY_PAGE_XML = "rss";
    private static final String DEFAULT_QUALITY = "480p";

    private final TorrentService torrentService;
    private final ImdbScraperService imdbScraperService;

    public AnimeProviderService(RestTemplate restTemplate,
                                PopcornProperties popcornConfig,
                                SettingsService settingsService,
                                TorrentService torrentService, ImdbScraperService imdbScraperService) {
        super(restTemplate);
        this.torrentService = torrentService;
        this.imdbScraperService = imdbScraperService;

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
        return imdbScraperService.retrievePage(genre, sortBy, page, keywords);

//        return invokeWithUriProvider(apiUri -> {
//            var uri = buildSearchRequestUri(apiUri, genre, sortBy, keywords, page);
//
//            log.debug("Retrieving anime provider page \"{}\"", uri);
//            var response = restTemplate.getForEntity(uri, String.class);
//
//            if (response.hasBody()) {
//                var document = parseXmlResponse(response.getBody());
//                var items = document.getChannel().getItems();
//
//                if (items != null) {
//                    return new PageImpl<>(items.stream()
//                            .map(e -> Anime.builder()
//                                    .nyaaId(e.getTitle())
//                                    .imdbId(IdParser.extractId(e.getGuid()))
//                                    .title(TitleParser.normaliseTitle(e.getTitle()))
//                                    .year(DateParser.convertDateToYear(e.getPubDate()))
//                                    .build())
//                            .collect(Collectors.toList()));
//                }
//            }
//
//            return Page.empty();
//        });
    }

    private Anime getDetailsInternal(String imdbId) {
        return invokeWithUriProvider(apiUri -> {
            var genre = new Genre(Genre.ALL_KEYWORD, "");
            var uri = buildSearchRequestUri(apiUri, genre, null, imdbId, 1);

            log.debug("Retrieving anime provider details of \"{}\"", uri);
            var response = restTemplate.getForEntity(uri, String.class);

            if (response.hasBody()) {
                var document = parseXmlResponse(response.getBody());
                var items = document.getChannel().getItems();

                if (items != null && items.size() == 1) {
                    var item = items.get(0);
                    var id = IdParser.extractId(item.getGuid());
                    var torrentInfo = retrieveTorrentData(item);

                    return Anime.builder()
                            .nyaaId(id)
                            .imdbId(id)
                            .title(TitleParser.normaliseTitle(item.getTitle()))
                            .year(DateParser.convertDateToYear(item.getPubDate()))
                            .episodes(extractEpisodesFromTorrentInfo(item.getLink(), torrentInfo))
                            .build();
                } else {
                    throw new MediaException("Could not find the details of the given media");
                }
            } else {
                throw new MediaException("No details response available for " + uri);
            }
        });
    }

    private TorrentInfo retrieveTorrentData(Item item) {
        try {
            return torrentService.getTorrentInfo(item.getLink()).get();
        } catch (InterruptedException | ExecutionException ex) {
            log.error(ex.getMessage(), ex);
            throw new MediaException("Failed to retrieve torrent data");
        }
    }

    private List<Episode> extractEpisodesFromTorrentInfo(String url, TorrentInfo torrentInfo) {
        return torrentInfo.getFiles().stream()
                .map(e -> createEpisode(url, e))
                .collect(Collectors.toList());
    }

    private Episode createEpisode(String url, TorrentFileInfo fileInfo) {
        var quality = QualityParser.extractQuality(fileInfo.getFilename())
                .orElse(DEFAULT_QUALITY);

        return Episode.builder()
                .title(TitleParser.normaliseTitle(fileInfo.getFilename()))
                .season(1)
                .episode(EpisodeParser.extractEpisode(fileInfo.getFilename())
                        .orElseGet(() -> fileInfo.getFileIndex() + 1))
                .torrents(Collections.singletonMap(quality, MediaTorrentInfo.builder()
                        .file(fileInfo.getFilename())
                        .url(url)
                        .build()))
                .build();
    }

    private static URI buildSearchRequestUri(URI apiUri, Genre genre, SortBy sortBy, String keywords, int page) {
        var uriBuilder = UriComponentsBuilder.fromUri(apiUri)
                .queryParam(QUERY_PAGE_TYPE_KEY, QUERY_PAGE_XML)
                .queryParam(QUERY_FILTER_KEY, 2)
                .queryParam(QUERY_GENRE_KEY, genreToQueryValue(genre))
                .queryParam(QUERY_PAGE_KEY, page);

        if (StringUtils.isNotEmpty(keywords)) {
            uriBuilder.queryParam(QUERY_SEARCH_KEY, keywords);
        }

        return uriBuilder.build(Collections.emptyMap());
    }

    private static Nyaa parseXmlResponse(String response) {
        try {
            var inputStream = new ByteArrayInputStream(response.getBytes(StandardCharsets.UTF_8));
            var context = JAXBContext.newInstance(Nyaa.class);

            return (Nyaa) context.createUnmarshaller().unmarshal(inputStream);
        } catch (JAXBException ex) {
            log.error(ex.getMessage(), ex);
            throw new MediaRetrievalException(null, ex.getMessage(), ex);
        }
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
