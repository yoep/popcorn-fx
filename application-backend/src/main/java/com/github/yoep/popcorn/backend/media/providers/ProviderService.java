package com.github.yoep.popcorn.backend.media.providers;

import com.github.yoep.popcorn.backend.media.filters.models.Category;
import com.github.yoep.popcorn.backend.media.filters.models.Genre;
import com.github.yoep.popcorn.backend.media.filters.models.SortBy;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import org.springframework.data.domain.Page;
import org.springframework.scheduling.annotation.Async;

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
    CompletableFuture<Page<T>> getPage(Genre genre, SortBy sortBy, int page);

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
    CompletableFuture<Page<T>> getPage(Genre genre, SortBy sortBy, int page, String keywords);

    /**
     * Get the {@link Media} details of the given imdb ID.
     *
     * @param imdbId The imdb id of the {@link Media}.
     * @return Returns the {@link Media} details for the given id.
     */
    @Async
    CompletableFuture<T> getDetails(String imdbId);

    /**
     * Retrieve the full details of the {@link Media} item.
     * This will load the details for the media item through the provider.
     *
     * @param media The media item to retrieve the details of.
     * @return Returns the retrieved media details.
     */
    @Async
    CompletableFuture<Media> retrieveDetails(Media media);
}
