package com.github.yoep.popcorn.activities;

import com.github.yoep.popcorn.models.Category;

public interface CategoryChangedActivity extends Activity {
    /**
     * Get the new selected category.
     *
     * @return Returns the new active category.
     */
    Category getCategory();
}
