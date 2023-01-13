package com.github.yoep.popcorn.backend.media.favorites;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFxInstance;
import com.github.yoep.popcorn.backend.media.favorites.models.Favorable;
import com.github.yoep.popcorn.backend.media.providers.Favorite;
import com.github.yoep.popcorn.backend.media.providers.FavoritesSet;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;
import org.springframework.util.Assert;

import java.util.Collections;
import java.util.List;
import java.util.Optional;

@Slf4j
@Service
@RequiredArgsConstructor
public class FavoriteService {
    /**
     * Check if the given {@link Favorable} is liked by the user.
     *
     * @param favorable The favorable to check.
     * @return Returns true if the favorable is liked, else false.
     */
    public boolean isLiked(Favorable favorable) {
        Assert.notNull(favorable, "favorable cannot be null");
        return FxLib.INSTANCE.is_media_liked(PopcornFxInstance.INSTANCE.get(), Favorite.from(favorable));
    }

    /**
     * Get all the {@link Favorable} items that are liked by the user.
     *
     * @return Returns the list of liked items by the user.
     */
    public List<Favorable> getAll() {
        return Optional.ofNullable(FxLib.INSTANCE.retrieve_all_favorites(PopcornFxInstance.INSTANCE.get()))
                .map(FavoritesSet::<Favorable>getAll)
                .orElse(Collections.emptyList());
    }

    /**
     * Add the given {@link Favorable} to the favorites.
     *
     * @param favorable The favorable to add.
     */
    public void addToFavorites(Favorable favorable) {
        Assert.notNull(favorable, "favorable cannot be null");
        var favorite = Favorite.from(favorable);
        FxLib.INSTANCE.add_to_favorites(PopcornFxInstance.INSTANCE.get(), favorite);
    }

    /**
     * Remove the given favorable from favorites.
     *
     * @param favorable The favorable to remove.
     */
    public void removeFromFavorites(Favorable favorable) {
        Assert.notNull(favorable, "favorable cannot be null");
        var favorite = Favorite.from(favorable);
        FxLib.INSTANCE.remove_from_favorites(PopcornFxInstance.INSTANCE.get(), favorite);
    }
}
