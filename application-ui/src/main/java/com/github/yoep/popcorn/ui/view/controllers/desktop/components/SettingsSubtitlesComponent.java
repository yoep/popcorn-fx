package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationSettings;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Subtitle;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.utils.LocaleText;
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
import java.util.concurrent.CompletableFuture;

@Slf4j
@RequiredArgsConstructor
public class SettingsSubtitlesComponent implements Initializable {
    private final ApplicationConfig applicationConfig;
    private final LocaleText localeText;

    private final DirectoryChooser cacheChooser = new DirectoryChooser();

    @FXML
    ComboBox<Subtitle.Language> defaultSubtitle;
    @FXML
    ComboBox<ApplicationSettings.SubtitleSettings.Family> fontFamily;
    @FXML
    ComboBox<ApplicationSettings.SubtitleSettings.DecorationType> decoration;
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
        var settings = getSettings();

//        defaultSubtitle.getItems().addAll(SubtitleLanguage.values());
//        defaultSubtitle.getSelectionModel().select(settings.getDefaultSubtitle());
//        defaultSubtitle.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> {
//            settings.setDefaultSubtitle(newValue);
//            applicationConfig.update(settings);
//        });
    }

    private void initializeFontFamily() {
        var settings = getSettings();

//        fontFamily.getItems().addAll(SubtitleFamily.values());
//        fontFamily.getSelectionModel().select(settings.getFontFamily());
//        fontFamily.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> {
//            settings.setFontFamily(newValue);
//            applicationConfig.update(settings);
//        });
    }

    private void initializeDecoration() {
        var settings = getSettings();

        decoration.setCellFactory(param -> getDecorationCell());
        decoration.setButtonCell(getDecorationCell());

//        decoration.getItems().addAll(DecorationType.values());
//        decoration.getSelectionModel().select(settings.getDecoration());
//        decoration.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> {
//            settings.setDecoration(newValue);
//            applicationConfig.update(settings);
//        });
    }

    private void initializeSize() {
        var settings = getSettings();

//        fontSize.getItems().addAll(SubtitleSettings.supportedFontSizes());
//        fontSize.getSelectionModel().select((Integer) settings.getFontSize());
//        fontSize.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> {
//            settings.setFontSize(newValue);
//            applicationConfig.update(settings);
//        });
    }

    private void initializeBold() {
        var settings = getSettings();

//        fontBold.setSelected(settings.isBold());
//        fontBold.selectedProperty().addListener((observable, oldValue, newValue) -> {
//            settings.setBold(newValue);
//            applicationConfig.update(settings);
//        });
    }

    private void initializeCacheDirectory() {
        getSettings().whenComplete((settings, throwable) -> {
            if (throwable == null) {
                var directory = settings.getDirectory();

                cacheChooser.setInitialDirectory(new File(directory));
                cacheDirectory.setText(directory);
                cacheDirectory.textProperty().addListener((observable, oldValue, newValue) -> {
                    var newDirectory = new File(newValue);

                    if (newDirectory.isDirectory()) {
                        applicationConfig.update(ApplicationSettings.SubtitleSettings.newBuilder(settings)
                                .setDirectory(newValue)
                                .build());
                        cacheChooser.setInitialDirectory(newDirectory);
                    }
                });
            } else {
                log.error("Failed to retrieve settings", throwable);
            }
        });
    }

    private void initializeClearCache() {
        getSettings().whenComplete((settings, throwable) -> {
            if (throwable == null) {
                clearCache.setSelected(settings.getAutoCleaningEnabled());
            } else {
                log.error("Failed to retrieve settings", throwable);
            }
        });

        clearCache.selectedProperty().addListener((observable, oldValue, newValue) ->
                getSettings().whenComplete((settings, throwable) -> {
                    if (throwable == null) {
                        applicationConfig.update(ApplicationSettings.SubtitleSettings.newBuilder(settings)
                                .setAutoCleaningEnabled(newValue).build());
                    } else {
                        log.error("Failed to retrieve settings", throwable);
                    }
                }));
    }

    private CompletableFuture<ApplicationSettings.SubtitleSettings> getSettings() {
        return applicationConfig.getSettings().thenApply(ApplicationSettings::getSubtitleSettings);
    }

    private ListCell<ApplicationSettings.SubtitleSettings.DecorationType> getDecorationCell() {
        return new ListCell<>() {
            @Override
            protected void updateItem(ApplicationSettings.SubtitleSettings.DecorationType item, boolean empty) {
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
