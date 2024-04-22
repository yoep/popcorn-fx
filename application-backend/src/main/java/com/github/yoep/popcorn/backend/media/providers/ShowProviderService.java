package com.github.yoep.popcorn.backend.media.providers;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.media.MediaItem;
import com.github.yoep.popcorn.backend.media.MediaResult;
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

import java.util.Collections;
import java.util.List;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutorService;

@Slf4j
@RequiredArgsConstructor
public class ShowProviderService implements ProviderService<ShowOverview> {
    private static final Category CATEGORY = Category.SERIES;

    private final FxLib fxLib;
    private final PopcornFx instance;
    private final ExecutorService executorService;

    @Override
    public boolean supports(Category category) {
        return category == CATEGORY;
    }

    @Override
    public CompletableFuture<List<ShowOverview>> getPage(Genre genre, SortBy sortBy, int page) {
        return CompletableFuture.supplyAsync(() -> getPage(genre, sortBy, StringUtils.EMPTY, page), executorService);
    }

    @Override
    public CompletableFuture<List<ShowOverview>> getPage(Genre genre, SortBy sortBy, int page, String keywords) {
        return CompletableFuture.supplyAsync(() -> getPage(genre, sortBy, keywords, page), executorService);
    }

    @Override
    public CompletableFuture<Media> retrieveDetails(Media media) {
        return CompletableFuture.supplyAsync(() -> {
            try {
                return getDetailsInternal(media);
            } catch (Exception ex) {
                throw new MediaDetailsException(media, "Failed to load show details", ex);
            }
        }, executorService);
    }

    @Override
    public void resetApiAvailability() {
        fxLib.reset_show_apis(instance);
    }

    public List<ShowOverview> getPage(Genre genre, SortBy sortBy, String keywords, int page) {
        try (var mediaResult = fxLib.retrieve_available_shows(instance, genre, sortBy, keywords, page)) {
            if (mediaResult.getTag() == MediaSetResult.Tag.Ok) {
                var shows = Optional.ofNullable(mediaResult.getUnion())
                        .map(MediaSetResult.MediaSetResultUnion::getOk)
                        .map(MediaSetResult.OkBody::getMediaSet)
                        .map(MediaSet::getShows)
                        .orElse(Collections.emptyList());
                log.debug("Retrieved shows {}", shows);

                return shows;
            } else {
                var mediaError = mediaResult.getUnion().getErr().getMediaError();
                switch (mediaError) {
                    case NoAvailableProviders -> throw new MediaRetrievalException(mediaError.getMessage());
                    case NoItemsFound -> {
                        return Collections.emptyList();
                    }
                    default -> throw new MediaException(mediaError.getMessage());
                }
            }
        }
    }

    private ShowDetails getDetailsInternal(Media media) {
        var result = fxLib.retrieve_media_details(instance, MediaItem.from(media));
        log.debug("Retrieved media details result {}", result);

        if (result.getTag() == MediaResult.Tag.Ok) {
            var mediaItem = result.getUnion().getOk().getMediaItem();
            return (ShowDetails) mediaItem.getMedia();
        } else {
            var error = result.getUnion().getErr();
            switch (error.getMediaError()) {
                case NoAvailableProviders -> throw new MediaRetrievalException("no providers are available");
                default -> throw new MediaException("failed to retrieve media details");
            }
        }
    }
}
