package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.media.filters.model.Category;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import javafx.scene.control.ListCell;

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

    protected ListCell<Category> createStartScreenCell() {
        return new ListCell<>() {
            @Override
            protected void updateItem(Category item, boolean empty) {
                super.updateItem(item, empty);

                if (!empty) {
                    setText(localeText.get("filter_" + item.name().toLowerCase()));
                } else {
                    setText(null);
                }
            }
        };
    }
}
