package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.settings.models.SubtitleSettings;
import com.github.yoep.popcorn.backend.settings.models.subtitles.DecorationType;
import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleFamily;
import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleLanguage;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.Node;
import javafx.scene.control.CheckBox;
import javafx.scene.control.ComboBox;
import javafx.scene.control.ListCell;
import javafx.scene.control.TextField;
import javafx.scene.input.MouseEvent;
import javafx.stage.DirectoryChooser;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.io.File;
import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
@RequiredArgsConstructor
public class SettingsSubtitlesComponent implements Initializable {
    private final ApplicationConfig applicationConfig;
    private final LocaleText localeText;

    private final DirectoryChooser cacheChooser = new DirectoryChooser();

    @FXML
    ComboBox<SubtitleLanguage> defaultSubtitle;
    @FXML
    ComboBox<SubtitleFamily> fontFamily;
    @FXML
    ComboBox<DecorationType> decoration;
    @FXML
    ComboBox<Integer> fontSize;
    @FXML
    CheckBox fontBold;
    @FXML
    TextField cacheDirectory;
    @FXML
    CheckBox clearCache;

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeDefaultSubtitle();
        initializeFontFamily();
        initializeDecoration();
        initializeSize();
        initializeBold();
        initializeCacheDirectory();
        initializeClearCache();
    }

    private void initializeDefaultSubtitle() {
        SubtitleSettings settings = getSettings();

        defaultSubtitle.getItems().addAll(SubtitleLanguage.values());
        defaultSubtitle.getSelectionModel().select(settings.getDefaultSubtitle());
        defaultSubtitle.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> {
            settings.setDefaultSubtitle(newValue);
            applicationConfig.update(settings);
        });
    }

    private void initializeFontFamily() {
        SubtitleSettings settings = getSettings();

        fontFamily.getItems().addAll(SubtitleFamily.values());
        fontFamily.getSelectionModel().select(settings.getFontFamily());
        fontFamily.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> {
            settings.setFontFamily(newValue);
            applicationConfig.update(settings);
        });
    }

    private void initializeDecoration() {
        SubtitleSettings settings = getSettings();

        decoration.setCellFactory(param -> getDecorationCell());
        decoration.setButtonCell(getDecorationCell());

        decoration.getItems().addAll(DecorationType.values());
        decoration.getSelectionModel().select(settings.getDecoration());
        decoration.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> {
            settings.setDecoration(newValue);
            applicationConfig.update(settings);
        });
    }

    private void initializeSize() {
        SubtitleSettings settings = getSettings();

        fontSize.getItems().addAll(SubtitleSettings.supportedFontSizes());
        fontSize.getSelectionModel().select((Integer) settings.getFontSize());
        fontSize.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> {
            settings.setFontSize(newValue);
            applicationConfig.update(settings);
        });
    }

    private void initializeBold() {
        SubtitleSettings settings = getSettings();

        fontBold.setSelected(settings.isBold());
        fontBold.selectedProperty().addListener((observable, oldValue, newValue) -> {
            settings.setBold(newValue);
            applicationConfig.update(settings);
        });
    }

    private void initializeCacheDirectory() {
        var settings = getSettings();
        var directory = settings.getDirectory();

        cacheChooser.setInitialDirectory(new File(directory));
        cacheDirectory.setText(directory);
        cacheDirectory.textProperty().addListener((observable, oldValue, newValue) -> {
            var newDirectory = new File(newValue);

            if (newDirectory.isDirectory()) {
                settings.setDirectory(newValue);
                applicationConfig.update(settings);
                cacheChooser.setInitialDirectory(newDirectory);
            }
        });
    }

    private void initializeClearCache() {
        var settings = getSettings();

        clearCache.setSelected(settings.isAutoCleaningEnabled());
        clearCache.selectedProperty().addListener((observable, oldValue, newValue) ->
        {
            settings.setAutoCleaningEnabled(newValue);
            applicationConfig.update(settings);
        });
    }

    private SubtitleSettings getSettings() {
        return applicationConfig.getSettings().getSubtitleSettings();
    }

    private ListCell<DecorationType> getDecorationCell() {
        return new ListCell<>() {
            @Override
            protected void updateItem(DecorationType item, boolean empty) {
                super.updateItem(item, empty);

                if (!empty) {
                    setText(localeText.get("settings_subtitles_style_" + item.toString().toLowerCase()));
                }
            }
        };
    }

    @FXML
    private void onCacheDirectoryClicked(MouseEvent event) {
        Node node = (Node) event.getSource();
        File newDirectory = cacheChooser.showDialog(node.getScene().getWindow());

        if (newDirectory != null && newDirectory.isDirectory()) {
            cacheDirectory.setText(newDirectory.getAbsolutePath());
        }
    }
}
