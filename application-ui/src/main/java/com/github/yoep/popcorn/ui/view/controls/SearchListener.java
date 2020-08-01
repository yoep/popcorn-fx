package com.github.yoep.popcorn.ui.view.controls;

public interface SearchListener {
    /**
     * Invoked when the search value is being changed.
     *
     * @param newValue The new search value.
     */
    void onSearchValueChanged(String newValue);

    /**
     * Invoked when the search value is cleared (manually or by the x icon).
     */
    void onSearchValueCleared();
}
