package com.github.yoep.popcorn.backend.media.providers;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.media.FavoritesSet;
import com.github.yoep.popcorn.backend.media.MediaItem;
import com.github.yoep.popcorn.backend.media.MediaResult;
import com.github.yoep.popcorn.backend.media.filters.model.Category;
import com.github.yoep.popcorn.backend.media.filters.model.Genre;
import com.github.yoep.popcorn.backend.media.filters.model.SortBy;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.util.Collections;
import java.util.List;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutorService;

@Slf4j
@RequiredArgsConstructor
public class FavoriteProviderService implements ProviderService<Media> {
    private static final Category CATEGORY = Category.FAVORITES;

    private final FxLib fxLib;
    private final PopcornFx instance;
    private final ExecutorService executorService;

    @Override
    public boolean supports(Category category) {
        return category == CATEGORY;
    }

    @Override
    public CompletableFuture<List<Media>> getPage(Genre genre, SortBy sortBy, int page) {
        return CompletableFuture.supplyAsync(() -> doInternalPageRetrieval(genre, sortBy, "", page), executorService);
    }

    @Override
    public CompletableFuture<List<Media>> getPage(Genre genre, SortBy sortBy, int page, String keywords) {
        return CompletableFuture.supplyAsync(() -> doInternalPageRetrieval(genre, sortBy, keywords, page), executorService);
    }

    @Override
    public CompletableFuture<Media> retrieveDetails(Media media) {
        return CompletableFuture.supplyAsync(() -> {
            try (var result = fxLib.retrieve_media_details(instance, MediaItem.from(media))) {
                if (result.getTag() == MediaResult.Tag.Ok) {
                    var mediaItem = result.getUnion().getOk().getMediaItem();
                    return mediaItem.getMedia();
                } else {
                    var error = result.getUnion().getErr();
                    switch (error.getMediaError()) {
                        case NoAvailableProviders -> throw new MediaRetrievalException("no providers are available");
                        default -> throw new MediaException("failed to retrieve media details");
                    }
                }
            }
        }, executorService);
    }

    @Override
    public void resetApiAvailability() {
        // no-op
    }

    private List<Media> doInternalPageRetrieval(Genre genre, SortBy sortBy, String keywords, int page) {
        try (var favorites = fxLib.retrieve_available_favorites(instance, genre, sortBy, keywords, page)) {
            return Optional.ofNullable(favorites)
                    .map(FavoritesSet::<Media>getAll)
                    .orElse(Collections.emptyList());
        }
    }
}
