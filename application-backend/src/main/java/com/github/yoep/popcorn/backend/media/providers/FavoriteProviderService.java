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
import org.springframework.data.domain.Page;
import org.springframework.data.domain.PageImpl;
import org.springframework.stereotype.Service;

import java.util.Collections;
import java.util.List;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;

@Slf4j
@Service
@RequiredArgsConstructor
public class FavoriteProviderService implements ProviderService<Media> {
    private static final Category CATEGORY = Category.FAVORITES;

    private final FxLib fxLib;
    private final PopcornFx instance;

    @Override
    public boolean supports(Category category) {
        return category == CATEGORY;
    }

    @Override
    public CompletableFuture<Page<Media>> getPage(Genre genre, SortBy sortBy, int page) {
        return CompletableFuture.completedFuture(new PageImpl<>(doInternalPageRetrieval(genre, sortBy, "", page)));
    }

    @Override
    public CompletableFuture<Page<Media>> getPage(Genre genre, SortBy sortBy, int page, String keywords) {
        return CompletableFuture.completedFuture(new PageImpl<>(doInternalPageRetrieval(genre, sortBy, keywords, page)));
    }

    @Override
    public CompletableFuture<Media> retrieveDetails(Media media) {
        try (var result = fxLib.retrieve_media_details(instance, MediaItem.from(media))) {
            if (result.getTag() == MediaResult.Tag.Ok) {
                var mediaItem = result.getUnion().getOk().getMediaItem();
                return CompletableFuture.completedFuture(mediaItem.getMedia());
            } else {
                var error = result.getUnion().getErr();
                switch (error.getMediaError()) {
                    case NoAvailableProviders -> throw new MediaRetrievalException("no providers are available");
                    default -> throw new MediaException("failed to retrieve media details");
                }
            }
        }
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
