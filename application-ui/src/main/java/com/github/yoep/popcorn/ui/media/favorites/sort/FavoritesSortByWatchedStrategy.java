package com.github.yoep.popcorn.ui.media.favorites.sort;

import com.github.yoep.popcorn.ui.media.providers.models.Media;
import com.github.yoep.popcorn.ui.view.models.SortBy;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Component;
import org.springframework.util.Assert;

import java.util.stream.Stream;

@Slf4j
@Component
public class FavoritesSortByWatchedStrategy extends AbstractFavoriteSortStrategy {
    @Override
    public boolean support(SortBy sortBy) {
        Assert.notNull(sortBy, "sortBy cannot be null");
        return sortBy.getKey().equalsIgnoreCase("watched");
    }

    @Override
    public Stream<Media> sort(Stream<Media> mediaStream) {
        log.trace("Sorting favorites based on the watched state");
        return mediaStream.sorted(this::sortByWatchedState);
    }

    private int sortByWatchedState(Media o1, Media o2) {
        var typeSort = sortByType(o1, o2);

        if (typeSort != 0)
            return typeSort;

        // sort by the watched state of the media items
        if (o1.isWatched() && o2.isWatched())
            return 0;
        if (o1.isWatched())
            return 1;
        if (o2.isWatched())
            return -1;

        return 0;
    }
}
