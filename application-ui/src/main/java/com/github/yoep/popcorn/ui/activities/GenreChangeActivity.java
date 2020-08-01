package com.github.yoep.popcorn.ui.activities;

import com.github.yoep.popcorn.ui.view.models.Genre;

public interface GenreChangeActivity extends Activity {
    /**
     * Get the genre that has been selected.
     *
     * @return Returns the selected genre.
     */
    Genre getGenre();
}
