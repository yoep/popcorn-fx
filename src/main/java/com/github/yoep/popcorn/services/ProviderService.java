package com.github.yoep.popcorn.services;

import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.models.Genre;
import com.github.yoep.popcorn.models.SortBy;
import org.springframework.scheduling.annotation.Async;

import java.util.List;
import java.util.concurrent.CompletableFuture;

public interface ProviderService<T extends Media> {
    /**
     * Get the given page for this media provider service.
     *
     * @param genre  The genre of the media items that should be loaded.
     * @param sortBy The sort order of the media items.
     * @param page   The page to retrieve.
     * @return Returns the list of {@link Media} items for the given page.
     */
    @Async
    CompletableFuture<List<T>> getPage(Genre genre, SortBy sortBy, int page);
}