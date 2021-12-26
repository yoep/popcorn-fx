package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.settings.SettingsService;
import com.github.yoep.popcorn.backend.settings.models.StartScreen;
import javafx.scene.control.ListCell;
import org.springframework.context.ApplicationEventPublisher;

import java.util.Locale;

public abstract class AbstractSettingsUiComponent extends AbstractSettingsComponent {
    protected AbstractSettingsUiComponent(ApplicationEventPublisher eventPublisher, LocaleText localeText, SettingsService settingsService) {
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

    protected ListCell<StartScreen> createStartScreenCell() {
        return new ListCell<>() {
            @Override
            protected void updateItem(StartScreen item, boolean empty) {
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
