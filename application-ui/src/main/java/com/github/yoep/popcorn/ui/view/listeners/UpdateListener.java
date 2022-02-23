package com.github.yoep.popcorn.ui.view.listeners;

import com.github.yoep.popcorn.ui.updater.UpdateState;
import com.github.yoep.popcorn.ui.updater.VersionInfo;

public interface UpdateListener {
    /**
     * Invoked when the known update information is changed.
     *
     * @param newValue The new update information.
     */
    void onUpdateInfoChanged(VersionInfo newValue);

    /**
     * Invoked when the update information state is changed.
     *
     * @param newState The new update state.
     */
    void onUpdateStateChanged(UpdateState newState);
}
