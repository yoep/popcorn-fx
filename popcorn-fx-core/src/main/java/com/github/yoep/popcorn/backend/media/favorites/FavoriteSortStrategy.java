package com.github.yoep.popcorn.backend.media.favorites;

import com.github.yoep.popcorn.backend.media.filters.model.SortBy;
import com.github.yoep.popcorn.backend.media.providers.models.Media;

import java.util.stream.Stream;

public interface FavoriteSortStrategy {
    /**
     * Check if the strategy supports the given {@link SortBy}.
     *
     * @param sortBy The sort by the strategy needs to support.
     * @return Returns true if the strategy supports the {@link SortBy}, else false.
     */
    boolean support(SortBy sortBy);

    /**
     * Sort the given media stream.
     *
     * @param mediaStream The media stream to sort.
     * @return Returns the sorted media stream.
     */
    Stream<Media> sort(Stream<Media> mediaStream);
}
