package com.github.yoep.popcorn.backend.media.favorites.sort;

import com.github.yoep.popcorn.backend.media.favorites.FavoriteSortStrategy;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.providers.models.Movie;
import com.github.yoep.popcorn.backend.media.providers.models.ShowOverview;

public abstract class AbstractFavoriteSortStrategy implements FavoriteSortStrategy {
    protected int sortByType(Media o1, Media o2) {
        // make sure movies are always before the shows
        if (o1 instanceof Movie && o2 instanceof ShowOverview)
            return -1;
        if (o1 instanceof ShowOverview && o2 instanceof Movie)
            return 1;

        return 0;
    }
}
