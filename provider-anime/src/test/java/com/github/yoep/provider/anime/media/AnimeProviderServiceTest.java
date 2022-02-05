package com.github.yoep.provider.anime.media;

import com.github.yoep.popcorn.backend.adapters.torrent.TorrentService;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentFileInfo;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentInfo;
import com.github.yoep.popcorn.backend.config.properties.PopcornProperties;
import com.github.yoep.popcorn.backend.config.properties.ProviderProperties;
import com.github.yoep.popcorn.backend.media.filters.models.Category;
import com.github.yoep.popcorn.backend.media.filters.models.Genre;
import com.github.yoep.popcorn.backend.media.filters.models.SortBy;
import com.github.yoep.popcorn.backend.media.providers.models.Episode;
import com.github.yoep.popcorn.backend.media.providers.models.MediaTorrentInfo;
import com.github.yoep.popcorn.backend.settings.SettingsService;
import com.github.yoep.popcorn.backend.settings.models.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.models.ServerSettings;
import com.github.yoep.provider.anime.media.models.Anime;
import org.apache.commons.io.IOUtils;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.core.io.ClassPathResource;
import org.springframework.http.ResponseEntity;
import org.springframework.web.client.RestTemplate;

import java.io.IOException;
import java.net.URI;
import java.net.URISyntaxException;
import java.nio.charset.StandardCharsets;
import java.text.MessageFormat;
import java.util.Collections;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutionException;

import static org.junit.jupiter.api.Assertions.*;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class AnimeProviderServiceTest {
    @Mock
    private RestTemplate restTemplate;
    @Mock
    private PopcornProperties popcornConfig;
    @Mock
    private SettingsService settingsService;
    @Mock
    private ServerSettings serverSettings;
    @Mock
    private ProviderProperties providerProperties;
    @Mock
    private TorrentService torrentService;

    private AnimeProviderService service;

    @BeforeEach
    void setUp() {
        var applicationSettings = ApplicationSettings.builder()
                .serverSettings(serverSettings)
                .build();

        when(settingsService.getSettings()).thenReturn(applicationSettings);
        when(popcornConfig.getProvider(Category.ANIME.getProviderName())).thenReturn(providerProperties);

        service = new AnimeProviderService(restTemplate, popcornConfig, settingsService, torrentService);
    }

    @Test
    void testSupports_whenCategoryIsAnime_shouldReturnTrue() {
        var category = Category.ANIME;

        var result = service.supports(category);

        assertTrue(result);
    }

    @Test
    void testSupports_whenCategoryIsMovie_shouldReturnFalse() {
        var category = Category.MOVIES;

        var result = service.supports(category);

        assertFalse(result);
    }

    @Test
    void testGetPage_whenGenreIsAnimeMusicVideo_shouldRetrieveAnimeDataForTheGenre()
            throws URISyntaxException, IOException, ExecutionException, InterruptedException {
        var genre = new Genre("anime-music-video", "lorem");
        var sortBy = new SortBy("popular", "ipsum");
        var page = 1;
        var response = getResourceHtml("overview.xml");
        var uri = "https://www.default-api.com";
        var expectedUri = new URI(MessageFormat.format("{0}?page=rss&f={1}&c={2}&p={3}", uri, 2, "1_1", page));
        when(providerProperties.getUris()).thenReturn(Collections.singletonList(new URI(uri)));
        when(restTemplate.getForEntity(isA(URI.class), eq(String.class))).thenReturn(ResponseEntity.ok(response));
        service = new AnimeProviderService(restTemplate, popcornConfig, settingsService, torrentService);

        var completableFuture = service.getPage(genre, sortBy, page);

        verify(restTemplate).getForEntity(expectedUri, String.class);
        var result = completableFuture.get();
        assertEquals(2, result.getTotalElements());
    }

    @Test
    void testGetPage_whenGenreIsEnglishTranslated_shouldRetrieveAnimeDataForTheGenre()
            throws URISyntaxException, IOException, ExecutionException, InterruptedException {
        var genre = new Genre("english-translated", "lorem");
        var sortBy = new SortBy("popular", "ipsum");
        var page = 1;
        var response = getResourceHtml("overview.xml");
        var uri = "https://www.default-api.com";
        var expectedUri = new URI(MessageFormat.format("{0}?page=rss&f={1}&c={2}&p={3}", uri, 2, "1_2", page));
        when(providerProperties.getUris()).thenReturn(Collections.singletonList(new URI(uri)));
        when(restTemplate.getForEntity(isA(URI.class), eq(String.class))).thenReturn(ResponseEntity.ok(response));
        service = new AnimeProviderService(restTemplate, popcornConfig, settingsService, torrentService);

        var completableFuture = service.getPage(genre, sortBy, page);

        verify(restTemplate).getForEntity(expectedUri, String.class);
        var result = completableFuture.get();
        assertEquals(2, result.getTotalElements());
    }

    @Test
    void testGetDetails_whenIdIsGiven_shouldRetrieveTheDetailsFromTheSiteAndTorrent()
            throws URISyntaxException, IOException, ExecutionException, InterruptedException {
        var detailId = "My details title";
        var id = "589001";
        var response = getResourceHtml("details.xml");
        var uri = "https://www.default-api.com";
        var torrentInfo = mock(TorrentInfo.class);
        var torrentFile = mock(TorrentFileInfo.class);
        var expectedTorrentUri = "https://nyaa.si/download/my.torrent";
        var expectedResult = Anime.builder()
                .nyaaId(id)
                .imdbId(id)
                .title(detailId)
                .year("2022")
                .episodes(Collections.singletonList(Episode.builder()
                        .title("my filename")
                        .episode(1)
                        .season(1)
                        .torrents(Collections.singletonMap("720p", MediaTorrentInfo.builder()
                                        .file("my filename [720p]")
                                        .url("https://nyaa.si/download/my.torrent")
                                .build()))
                        .build()))
                .build();
        when(providerProperties.getUris()).thenReturn(Collections.singletonList(new URI(uri)));
        when(restTemplate.getForEntity(isA(URI.class), eq(String.class))).thenReturn(ResponseEntity.ok(response));
        when(torrentService.getTorrentInfo(expectedTorrentUri)).thenReturn(CompletableFuture.completedFuture(torrentInfo));
        when(torrentInfo.getFiles()).thenReturn(Collections.singletonList(torrentFile));
        when(torrentFile.getFilename()).thenReturn("my filename [720p]");
        service = new AnimeProviderService(restTemplate, popcornConfig, settingsService, torrentService);

        var completableFuture = service.getDetails(detailId);

        verify(torrentService).getTorrentInfo(expectedTorrentUri);
        var result = completableFuture.get();
        assertEquals(expectedResult, result);
    }

    private String getResourceHtml(String filename) throws IOException {
        var resource = new ClassPathResource(filename);

        return IOUtils.toString(resource.getInputStream(), StandardCharsets.UTF_8);
    }
}