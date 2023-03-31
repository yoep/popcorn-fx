package com.github.yoep.popcorn.backend.media.providers;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.media.MediaError;
import com.github.yoep.popcorn.backend.media.MediaSet;
import com.github.yoep.popcorn.backend.media.MediaSetResult;
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
    private final PopcornFx instance;

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
        fxLib.reset_show_apis(instance);
    }

    public Page<ShowOverview> getPage(Genre genre, SortBy sortBy, String keywords, int page) {
        try (var mediaResult = fxLib.retrieve_available_shows(instance, genre, sortBy, keywords, page)) {
            if (mediaResult.getTag() == MediaSetResult.Tag.Ok) {
                var shows = Optional.ofNullable(mediaResult.getUnion())
                        .map(MediaSetResult.MediaSetResultUnion::getOk)
                        .map(MediaSetResult.OkBody::getMediaSet)
                        .map(MediaSet::getShows)
                        .orElse(Collections.emptyList());
                log.debug("Retrieved shows {}", shows);

                return new PageImpl<>(shows);
            } else {
                var mediaError = mediaResult.getUnion().getErr().getMediaError();
                if (mediaError == MediaError.NoAvailableProviders) {
                    throw new MediaRetrievalException(mediaError.getMessage());
                } else {
                    throw new MediaException(mediaError.getMessage());
                }
            }
        }
    }

    private ShowDetails getDetailsInternal(String imdbId) {
        return fxLib.retrieve_show_details(instance, imdbId);
    }
}
