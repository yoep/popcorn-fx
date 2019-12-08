package com.github.yoep.popcorn.services;

import com.github.yoep.popcorn.providers.media.models.Media;
import org.springframework.scheduling.annotation.Async;

import java.util.List;
import java.util.concurrent.CompletableFuture;

public interface ProviderService<T extends Media> {
    /**
     * Get the given page for this media provider service.
     *
     * @param page The page to retrieve.
     * @return Returns the list of {@link Media} items for the given page.
     */
    @Async
    CompletableFuture<List<T>> getPage(int page);
}
