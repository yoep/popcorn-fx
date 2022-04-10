package com.github.yoep.popcorn.backend.media.favorites.sort;

import com.github.yoep.popcorn.backend.media.filters.model.SortBy;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Component;
import org.springframework.util.Assert;

import java.util.stream.Stream;

@Slf4j
@Component
public class FavoritesSortByTitleStrategy extends AbstractFavoriteSortStrategy {

    @Override
    public boolean support(SortBy sortBy) {
        Assert.notNull(sortBy, "sortBy cannot be null");
        return sortBy.getKey().equalsIgnoreCase("title");
    }

    @Override
    public Stream<Media> sort(Stream<Media> mediaStream) {
        log.trace("Sorting favorites based on the title");
        return mediaStream.sorted(this::sortByTitle);
    }

    private int sortByTitle(Media o1, Media o2) {
        var typeSort = sortByType(o1, o2);

        if (typeSort != 0)
            return typeSort;

        return o1.getTitle().compareTo(o2.getTitle());
    }
}
