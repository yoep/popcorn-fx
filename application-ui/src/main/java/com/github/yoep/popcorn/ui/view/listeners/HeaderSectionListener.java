package com.github.yoep.popcorn.ui.view.listeners;

public interface HeaderSectionListener {
    /**
     * Invoked when the update available state is changed.
     *
     * @param isUpdateAvailable Indication if an update is available.
     */
    void onUpdateAvailableChanged(boolean isUpdateAvailable);
}
