package com.github.yoep.popcorn.ui.media.favorites.sort;

import com.github.yoep.popcorn.ui.media.favorites.FavoriteSortStrategy;
import com.github.yoep.popcorn.ui.media.providers.models.Media;
import com.github.yoep.popcorn.ui.media.providers.models.Movie;
import com.github.yoep.popcorn.ui.media.providers.models.Show;

public abstract class AbstractFavoriteSortStrategy implements FavoriteSortStrategy {
    protected int sortByType(Media o1, Media o2) {
        // make sure movies are always before the shows
        if (o1 instanceof Movie && o2 instanceof Show)
            return -1;
        if (o1 instanceof Show && o2 instanceof Movie)
            return 1;

        return 0;
    }
}
