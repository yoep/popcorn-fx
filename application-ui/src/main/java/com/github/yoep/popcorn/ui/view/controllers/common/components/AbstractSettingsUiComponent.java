package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationSettings;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import javafx.scene.control.ListCell;

import java.util.Arrays;
import java.util.Locale;

public abstract class AbstractSettingsUiComponent extends AbstractSettingsComponent {
    protected AbstractSettingsUiComponent(EventPublisher eventPublisher, LocaleText localeText, ApplicationConfig settingsService) {
        super(eventPublisher, localeText, settingsService);
    }

    protected ListCell<Locale> createLanguageCell() {
        return new ListCell<>() {
            @Override
            protected void updateItem(Locale item, boolean empty) {
                super.updateItem(item, empty);

                if (!empty) {
                    setText(localeText.get("language_" + item.getLanguage()));
                } else {
                    setText(null);
                }
            }
        };
    }

    protected ListCell<ApplicationSettings.UISettings.Scale> createUiScaleCell() {
        return new ListCell<>() {
            @Override
            protected void updateItem(ApplicationSettings.UISettings.Scale item, boolean empty) {
                super.updateItem(item, empty);

                if (!empty) {
                    var percentage = (int) (item.getFactor() * 100);
                    setText(percentage + "%");
                } else {
                    setText(null);
                }
            }
        };
    }

    protected ListCell<Media.Category> createStartScreenCell() {
        return new ListCell<>() {
            @Override
            protected void updateItem(Media.Category item, boolean empty) {
                super.updateItem(item, empty);

                if (!empty) {
                    setText(localeText.get("filter_" + item.name().toLowerCase()));
                } else {
                    setText(null);
                }
            }
        };
    }

    protected static Media.Category[] startScreens() {
        return Arrays.stream(Media.Category.values())
                .filter(e -> e != Media.Category.UNRECOGNIZED)
                .toArray(Media.Category[]::new);
    }
}
