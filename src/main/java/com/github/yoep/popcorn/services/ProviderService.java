package com.github.yoep.popcorn.services;

import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.models.Category;
import com.github.yoep.popcorn.models.Genre;
import com.github.yoep.popcorn.models.SortBy;
import org.springframework.scheduling.annotation.Async;

import java.util.List;
import java.util.concurrent.CompletableFuture;

public interface ProviderService<T extends Media> {
    /**
     * Check if this {@link ProviderService} supports the given category.
     *
     * @param category The category that should be supported.
     * @return Returns true if this provider supports the given category, else false.
     */
    boolean supports(Category category);

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

    /**
     * Get the given page with search criteria for this media provider service.
     *
     * @param genre    The genre of the media items that should be loaded.
     * @param sortBy   The sort order of the media items.
     * @param page     The page to retrieve.
     * @param keywords The search keywords to search on.
     * @return Returns the list of {@link Media} items for the given page.
     */
    @Async
    CompletableFuture<List<T>> getPage(Genre genre, SortBy sortBy, int page, String keywords);

    /**
     * Show the details of the given media item.
     *
     * @param media The media item to show the details of.
     */
    void showDetails(Media media);
}
