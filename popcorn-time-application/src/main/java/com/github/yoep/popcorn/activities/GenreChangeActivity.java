package com.github.yoep.popcorn.activities;

import com.github.yoep.popcorn.models.Genre;

public interface GenreChangeActivity extends Activity {
    /**
     * Get the genre that has been selected.
     *
     * @return Returns the selected genre.
     */
    Genre getGenre();
}
