package com.github.yoep.popcorn.backend.media.providers;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media.Category;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media.Genre;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media.SortBy;
import com.github.yoep.popcorn.backend.media.Media;

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
    CompletableFuture<List<T>> getPage(Category category, Genre genre, SortBy sortBy, int page);

    /**
     * Get the given page with search criteria for this media provider service.
     *
     * @param genre    The genre of the media items that should be loaded.
     * @param sortBy   The sort order of the media items.
     * @param page     The page to retrieve.
     * @param keywords The search keywords to search on.
     * @return Returns the list of {@link Media} items for the given page.
     */
    CompletableFuture<List<T>> getPage(Category category, Genre genre, SortBy sortBy, int page, String keywords);

    /**
     * Retrieve the full details of the {@link Media} item.
     * This will load the details for the media item through the provider.
     *
     * @param media The media item to retrieve the details of.
     * @return Returns the retrieved media details.
     */
    CompletableFuture<Media> retrieveDetails(Media media);

    /**
     * Reset the API availability.
     * This will allow each API to become available again and tested/invoked.
     */
    void resetApiAvailability(Category category);
}
