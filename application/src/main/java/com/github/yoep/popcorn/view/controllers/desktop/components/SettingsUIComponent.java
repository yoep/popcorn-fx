package com.github.yoep.popcorn.view.controllers.desktop.components;

import com.github.yoep.popcorn.settings.SettingsService;
import com.github.yoep.popcorn.settings.models.UIScale;
import com.github.yoep.popcorn.settings.models.UISettings;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.ComboBox;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Component;

import java.net.URL;
import java.util.ResourceBundle;

@Component
@RequiredArgsConstructor
public class SettingsUIComponent implements Initializable {
    private final SettingsService settingsService;

    @FXML
    private ComboBox defaultLanguage;
    @FXML
    private ComboBox<UIScale> uiScale;
    @FXML
    private ComboBox startScreen;

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeUIScale();
    }

    private void initializeUIScale() {
        uiScale.getItems().add(new UIScale(0.25f));
        uiScale.getItems().add(new UIScale(0.5f));
        uiScale.getItems().add(new UIScale(0.75f));
        uiScale.getItems().add(new UIScale(1.0f));
        uiScale.getItems().add(new UIScale(1.25f));
        uiScale.getItems().add(new UIScale(1.50f));
        uiScale.getItems().add(new UIScale(2.0f));
        uiScale.getItems().add(new UIScale(3.0f));

        uiScale.getSelectionModel().select(getUiSettings().getUiScale());
        uiScale.getSelectionModel().selectedItemProperty().addListener(((observable, oldValue, newValue) -> getUiSettings().setUiScale(newValue)));
    }

    private UISettings getUiSettings() {
        return settingsService.getSettings().getUiSettings();
    }
}
