package com.github.yoep.popcorn.backend.media.favorites.sort;

import com.github.yoep.popcorn.backend.media.filters.models.SortBy;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Component;
import org.springframework.util.Assert;

import java.util.stream.Stream;

@Slf4j
@Component
public class FavoritesSortByRatingStrategy extends AbstractFavoriteSortStrategy {

    @Override
    public boolean support(SortBy sortBy) {
        Assert.notNull(sortBy, "sortBy cannot be null");
        return sortBy.getKey().equalsIgnoreCase("rating");
    }

    @Override
    public Stream<Media> sort(Stream<Media> mediaStream) {
        log.trace("Sorting favorites based on the rating");
        return mediaStream.sorted(this::sortByRating);
    }

    private int sortByRating(Media o1, Media o2) {
        var typeSort = sortByType(o1, o2);

        if (typeSort != 0)
            return typeSort;

        var rating1 = o1.getRating().getPercentage();
        var rating2 = o2.getRating().getPercentage();

        // make sure the highest rating is always first
        // so swap the ratings around during comparison
        return Integer.compare(rating2, rating1);
    }
}
