package com.github.yoep.popcorn.ui.view.controllers.desktop.sections;

import com.github.yoep.popcorn.ui.events.CloseSettingsEvent;
import javafx.fxml.FXML;
import lombok.RequiredArgsConstructor;
import org.springframework.context.ApplicationEventPublisher;

@RequiredArgsConstructor
public class SettingsSectionController {
    private final ApplicationEventPublisher eventPublisher;

    @FXML
    private void onClose() {
        eventPublisher.publishEvent(new CloseSettingsEvent(this));
    }
}
