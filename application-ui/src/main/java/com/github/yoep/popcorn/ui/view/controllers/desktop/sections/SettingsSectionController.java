package com.github.yoep.popcorn.ui.view.controllers.desktop.sections;

import com.github.yoep.popcorn.ui.events.ActivityManager;
import com.github.yoep.popcorn.ui.events.CloseSettingsEvent;
import javafx.fxml.FXML;
import lombok.RequiredArgsConstructor;

@RequiredArgsConstructor
public class SettingsSectionController {
    private final ActivityManager activityManager;

    @FXML
    private void onClose() {
        activityManager.register(new CloseSettingsEvent() {
        });
    }
}
