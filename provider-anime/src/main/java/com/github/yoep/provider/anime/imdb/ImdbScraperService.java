package com.github.yoep.provider.anime.imdb;

import com.github.yoep.popcorn.backend.media.filters.model.Genre;
import com.github.yoep.popcorn.backend.media.filters.model.SortBy;
import com.github.yoep.provider.anime.media.models.Anime;
import lombok.extern.slf4j.Slf4j;
import org.springframework.data.domain.Page;
import org.springframework.stereotype.Service;

@Slf4j
@Service
public record ImdbScraperService(TitleSearchScraper titleSearchScraper,
                                 DetailsScraper detailsScraper) {
    public Page<Anime> retrievePage(Genre genre, SortBy sortBy, int page, String keywords) {
        return titleSearchScraper.retrievePage(genre, sortBy, page, keywords);
    }

    public Anime retrieveDetails(String imdbId) {
        return detailsScraper.retrieveDetails(imdbId);
    }
}
