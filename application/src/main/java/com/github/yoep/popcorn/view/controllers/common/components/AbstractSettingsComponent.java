package com.github.yoep.popcorn.view.controllers.common.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.SuccessNotificationActivity;
import com.github.yoep.popcorn.messages.SettingsMessage;
import com.github.yoep.popcorn.settings.SettingsService;
import javafx.scene.control.TextFormatter;

public abstract class AbstractSettingsComponent {
    private static final int NOTIFICATION_TIME_BETWEEN = 750;

    protected final ActivityManager activityManager;
    protected final LocaleText localeText;
    protected final SettingsService settingsService;

    private long lastNotification;

    protected AbstractSettingsComponent(ActivityManager activityManager, LocaleText localeText, SettingsService settingsService) {
        this.activityManager = activityManager;
        this.localeText = localeText;
        this.settingsService = settingsService;
    }

    protected TextFormatter<Object> numericTextFormatter() {
        return new TextFormatter<>(change -> {
            var text = change.getText();

            if (text.matches("[0-9]*")) {
                return change;
            }

            return null;
        });
    }

    /**
     * Show the "settings saved" notification to the user.
     */
    protected void showNotification() {
        if (isNotificationAllowed()) {
            lastNotification = System.currentTimeMillis();
            activityManager.register((SuccessNotificationActivity) () -> localeText.get(SettingsMessage.SETTINGS_SAVED));
        }
    }

    //region Functions

    private boolean isNotificationAllowed() {
        return System.currentTimeMillis() - lastNotification > NOTIFICATION_TIME_BETWEEN;
    }

    //endregion
}
