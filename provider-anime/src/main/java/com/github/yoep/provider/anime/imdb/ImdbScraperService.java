package com.github.yoep.provider.anime.imdb;

import com.github.yoep.popcorn.backend.media.filters.models.Genre;
import com.github.yoep.popcorn.backend.media.filters.models.SortBy;
import com.github.yoep.provider.anime.media.models.Anime;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.data.domain.Page;
import org.springframework.stereotype.Service;

@Slf4j
@Service
@RequiredArgsConstructor
public class ImdbScraperService {
    private final TitleSearchScraper titleSearchScraper;

    public Page<Anime> retrievePage(Genre genre, SortBy sortBy, int page, String keywords) {
        return titleSearchScraper.retrievePage(genre, sortBy, page, keywords);
    }
}
