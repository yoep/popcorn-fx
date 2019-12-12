package com.github.yoep.popcorn.services;

import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.models.Genre;
import org.springframework.scheduling.annotation.Async;

import java.util.List;
import java.util.concurrent.CompletableFuture;

public interface ProviderService<T extends Media> {
    /**
     * Get the given page for this media provider service.
     *
     * @param genre The genre of the movies that should be loaded.
     * @param page  The page to retrieve.
     * @return Returns the list of {@link Media} items for the given page.
     */
    @Async
    CompletableFuture<List<T>> getPage(Genre genre, int page);
}
