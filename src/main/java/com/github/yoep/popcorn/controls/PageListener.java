package com.github.yoep.popcorn.controls;

/**
 * Listener that listens on page modifications.
 */
public interface PageListener {
    /**
     * Invoked when the page is being changed.
     *
     * @param previousPage The previous page that was loaded.
     * @param newPage The new page that should be loaded.
     */
    void onChange(int previousPage, int newPage);
}
