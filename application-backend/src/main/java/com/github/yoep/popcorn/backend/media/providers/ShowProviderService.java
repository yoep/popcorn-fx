package com.github.yoep.popcorn.backend.media.providers;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFxInstance;
import com.github.yoep.popcorn.backend.media.MediaSet;
import com.github.yoep.popcorn.backend.media.filters.model.Category;
import com.github.yoep.popcorn.backend.media.filters.model.Genre;
import com.github.yoep.popcorn.backend.media.filters.model.SortBy;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.providers.models.ShowDetails;
import com.github.yoep.popcorn.backend.media.providers.models.ShowOverview;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.lang3.StringUtils;
import org.springframework.data.domain.Page;
import org.springframework.data.domain.PageImpl;
import org.springframework.stereotype.Service;

import java.util.Collections;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;

@Slf4j
@Service
@RequiredArgsConstructor
public class ShowProviderService implements ProviderService<ShowOverview> {
    private static final Category CATEGORY = Category.SERIES;

    private final FxLib fxLib;

    @Override
    public boolean supports(Category category) {
        return category == CATEGORY;
    }

    @Override
    public CompletableFuture<Page<ShowOverview>> getPage(Genre genre, SortBy sortBy, int page) {
        return CompletableFuture.completedFuture(getPage(genre, sortBy, StringUtils.EMPTY, page));
    }

    @Override
    public CompletableFuture<Page<ShowOverview>> getPage(Genre genre, SortBy sortBy, int page, String keywords) {
        return CompletableFuture.completedFuture(getPage(genre, sortBy, keywords, page));
    }

    @Override
    public CompletableFuture<ShowOverview> getDetails(String imdbId) {
        return CompletableFuture.completedFuture(getDetailsInternal(imdbId));
    }

    @Override
    public CompletableFuture<Media> retrieveDetails(Media media) {
        try {
            return CompletableFuture.completedFuture(getDetailsInternal(media.getId()));
        } catch (Exception ex) {
            throw new MediaDetailsException(media, "Failed to load show details", ex);
        }
    }

    @Override
    public void resetApiAvailability() {
        fxLib.reset_show_apis(PopcornFxInstance.INSTANCE.get());
    }

    public Page<ShowOverview> getPage(Genre genre, SortBy sortBy, String keywords, int page) {
        var shows = Optional.ofNullable(fxLib.retrieve_available_shows(PopcornFxInstance.INSTANCE.get(), genre, sortBy, keywords, page))
                .map(MediaSet::getShows)
                .orElse(Collections.emptyList());
        log.debug("Retrieved shows {}", shows);

        return new PageImpl<>(shows);
    }

    private ShowDetails getDetailsInternal(String imdbId) {
        return fxLib.retrieve_show_details(PopcornFxInstance.INSTANCE.get(), imdbId);
    }
}
