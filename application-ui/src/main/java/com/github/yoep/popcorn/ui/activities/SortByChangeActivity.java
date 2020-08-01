package com.github.yoep.popcorn.ui.activities;

import com.github.yoep.popcorn.ui.view.models.SortBy;

public interface SortByChangeActivity extends Activity {
    /**
     * Get the sort by that has been selected.
     *
     * @return Returns the selected sort by.
     */
    SortBy getSortBy();
}
