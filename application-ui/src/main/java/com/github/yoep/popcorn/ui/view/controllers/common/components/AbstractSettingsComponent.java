package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.ui.events.SuccessNotificationEvent;
import com.github.yoep.popcorn.ui.messages.SettingsMessage;
import com.github.yoep.popcorn.ui.settings.SettingsService;
import javafx.scene.control.TextFormatter;
import lombok.AccessLevel;
import lombok.RequiredArgsConstructor;
import org.springframework.context.ApplicationEventPublisher;

@RequiredArgsConstructor(access = AccessLevel.PROTECTED)
public abstract class AbstractSettingsComponent {
    private static final int NOTIFICATION_TIME_BETWEEN = 750;

    protected final ApplicationEventPublisher eventPublisher;
    protected final LocaleText localeText;
    protected final SettingsService settingsService;

    private long lastNotification;

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
            eventPublisher.publishEvent(new SuccessNotificationEvent(this, localeText.get(SettingsMessage.SETTINGS_SAVED)));
        }
    }

    //region Functions

    private boolean isNotificationAllowed() {
        return System.currentTimeMillis() - lastNotification > NOTIFICATION_TIME_BETWEEN;
    }

    //endregion
}
