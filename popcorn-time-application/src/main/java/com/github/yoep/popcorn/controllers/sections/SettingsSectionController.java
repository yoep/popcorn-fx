package com.github.yoep.popcorn.controllers.sections;

import com.github.yoep.popcorn.activities.CloseSettingsActivity;
import com.github.yoep.popcorn.activities.ActivityManager;
import javafx.fxml.FXML;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Controller;

@Controller
@RequiredArgsConstructor
public class SettingsSectionController {
    private final ActivityManager activityManager;

    @FXML
    private void onClose() {
        activityManager.register(new CloseSettingsActivity() {
        });
    }
}
