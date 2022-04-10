package com.github.yoep.popcorn.backend.media.favorites.sort;

import com.github.yoep.popcorn.backend.media.filters.model.SortBy;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Component;
import org.springframework.util.Assert;

import java.util.stream.Stream;

@Slf4j
@Component
public class FavoritesSortByYearStrategy extends AbstractFavoriteSortStrategy {
    @Override
    public boolean support(SortBy sortBy) {
        Assert.notNull(sortBy, "sortBy cannot be null");
        return sortBy.getKey().equalsIgnoreCase("year");
    }

    @Override
    public Stream<Media> sort(Stream<Media> mediaStream) {
        log.trace("Sorting favorites based on the year");
        return mediaStream.sorted(this::sortByYear);
    }

    private int sortByYear(Media o1, Media o2) {
        var typeSort = sortByType(o1, o2);

        if (typeSort != 0)
            return typeSort;

        var year1 = toYear(o1.getYear());
        var year2 = toYear(o2.getYear());

        return Integer.compare(year2, year1);
    }

    private int toYear(String year) {
        try {
            return Integer.parseInt(year);
        } catch (NumberFormatException ex) {
            log.debug("Media year is invalid, " + ex.getMessage(), ex);
            return 1970;
        }
    }
}
