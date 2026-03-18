package com.github.yoep.popcorn.backend.settings;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationSettings;

public abstract class AbstractApplicationSettingsEventListener implements ApplicationSettingsEventListener {
    @Override
    public void onUiSettingsChanged(ApplicationSettings.UISettings settings) {
        // no-op
    }

    @Override
    public void onSubtitleSettingsChanged(ApplicationSettings.SubtitleSettings settings) {
        // no-op
    }

    @Override
    public void onTrackingSettingsChanged(ApplicationSettings.TrackingSettings settings) {
        // no-op
    }
}
