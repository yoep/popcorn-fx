package com.github.yoep.popcorn.ui.activities;

import com.github.yoep.popcorn.ui.view.models.Category;

/**
 * Activity indicating that the header category has been changed by the user.
 */
public interface CategoryChangedActivity extends Activity {
    /**
     * Get the new selected category.
     *
     * @return Returns the new active category.
     */
    Category getCategory();
}
