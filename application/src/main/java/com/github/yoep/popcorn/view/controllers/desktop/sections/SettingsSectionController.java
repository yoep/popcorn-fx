package com.github.yoep.popcorn.view.controllers.desktop.sections;

import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.CloseSettingsActivity;
import javafx.fxml.FXML;
import lombok.RequiredArgsConstructor;

@RequiredArgsConstructor
public class SettingsSectionController {
    private final ActivityManager activityManager;

    @FXML
    private void onClose() {
        activityManager.register(new CloseSettingsActivity() {
        });
    }
}
