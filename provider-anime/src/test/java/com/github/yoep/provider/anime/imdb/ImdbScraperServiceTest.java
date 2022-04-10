package com.github.yoep.provider.anime.imdb;

import com.github.yoep.popcorn.backend.media.filters.model.Genre;
import com.github.yoep.popcorn.backend.media.filters.model.SortBy;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.mockito.Mockito.verify;

@ExtendWith(MockitoExtension.class)
class ImdbScraperServiceTest {
    @Mock
    private TitleSearchScraper titleSearchScraper;
    @Mock
    private DetailsScraper detailsScraper;
    @InjectMocks
    private ImdbScraperService service;

    @Test
    void testRetrievePage_whenPageIsGiven_shouldInvokeTitleSearchScraper() {
        var genre = new Genre("genre_key", "my  genre");
        var sortBy = new SortBy("pop", "popularity");
        var page = 2;

        service.retrievePage(genre, sortBy, page, null);

        verify(titleSearchScraper).retrievePage(genre, sortBy, page, null);
    }

    @Test
    void testRetrieveDetails_whenInvoked_shouldInvokeDetailsScraper() {
        var imdbId = "tt4877555896";

       service.retrieveDetails(imdbId);

       verify(detailsScraper).retrieveDetails(imdbId);
    }
}