package com.github.yoep.popcorn.backend.media.providers;

import com.github.yoep.popcorn.backend.FxLibInstance;
import com.github.yoep.popcorn.backend.PopcornFxInstance;
import com.github.yoep.popcorn.backend.media.FavoritesSet;
import com.github.yoep.popcorn.backend.media.filters.model.Category;
import com.github.yoep.popcorn.backend.media.filters.model.Genre;
import com.github.yoep.popcorn.backend.media.filters.model.SortBy;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
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
public class FavoriteProviderService implements ProviderService<Media> {
    private static final Category CATEGORY = Category.FAVORITES;

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
    public CompletableFuture<Media> getDetails(String imdbId) {
        throw new UnsupportedOperationException();
    }

    @Override
    public CompletableFuture<Media> retrieveDetails(Media media) {
        return CompletableFuture.completedFuture(FxLibInstance.INSTANCE.get().retrieve_favorite_details(PopcornFxInstance.INSTANCE.get(), media.getId()).getMedia());
    }

    @Override
    public void resetApiAvailability() {
        // no-op
    }

    private List<Media> doInternalPageRetrieval(Genre genre, SortBy sortBy, String keywords, int page) {
        return Optional.ofNullable(FxLibInstance.INSTANCE.get().retrieve_available_favorites(PopcornFxInstance.INSTANCE.get(), genre, sortBy, keywords, page))
                .map(FavoritesSet::<Media>getAll)
                .orElse(Collections.emptyList());
    }
}
