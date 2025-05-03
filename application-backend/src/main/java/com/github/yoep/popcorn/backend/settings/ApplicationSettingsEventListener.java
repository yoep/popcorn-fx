package com.github.yoep.popcorn.backend.settings;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationSettings;

public interface ApplicationSettingsEventListener {
    /**
     * Invoked when the subtitle settings of the application have been changed,
     *
     * @param settings The new subtitle settings.
     */
    void onSubtitleSettingsChanged(ApplicationSettings.SubtitleSettings settings);

    /**
     * Invoked when the tracking settings of the application have been changed.
     *
     * @param settings The new tracking settings.
     */
    void onTrackingSettingsChanged(ApplicationSettings.TrackingSettings settings);
}
