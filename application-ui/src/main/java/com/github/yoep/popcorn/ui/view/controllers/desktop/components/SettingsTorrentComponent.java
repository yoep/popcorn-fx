package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.view.controllers.common.components.AbstractSettingsComponent;
import com.github.yoep.popcorn.ui.view.controls.DelayedTextField;
import com.github.yoep.popcorn.ui.view.services.TorrentSettingService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.Node;
import javafx.scene.control.ComboBox;
import javafx.scene.control.ListCell;
import javafx.scene.control.TextField;
import javafx.scene.input.MouseEvent;
import javafx.stage.DirectoryChooser;
import lombok.extern.slf4j.Slf4j;

import java.io.File;
import java.net.URL;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;

@Slf4j
public class SettingsTorrentComponent extends AbstractSettingsComponent implements Initializable {
    private final DirectoryChooser cacheChooser = new DirectoryChooser();

    private final TorrentSettingService torrentSettingService;

    @FXML
    DelayedTextField downloadLimit;
    @FXML
    DelayedTextField uploadLimit;
    @FXML
    DelayedTextField connectionLimit;
    @FXML
    TextField cacheDirectory;
    @FXML
    ComboBox<ApplicationSettings.TorrentSettings.CleaningMode> cleaningMode;

    public SettingsTorrentComponent(EventPublisher eventPublisher,
                                    LocaleText localeText,
                                    ApplicationConfig settingsService,
                                    TorrentSettingService torrentSettingService) {
        super(eventPublisher, localeText, settingsService);
        this.torrentSettingService = torrentSettingService;
    }

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeDownloadLimit();
        initializeUploadLimit();
        initializeConnectionLimit();
        initializeCacheDirectory();
        initializeCleaningMode();
    }

    private void initializeDownloadLimit() {
        var settings = getSettings();

//        downloadLimit.setTextFormatter(numericTextFormatter());
//        downloadLimit.setValue(torrentSettingService.toDisplayValue(settings.getDownloadRateLimit()));
//        downloadLimit.valueProperty().addListener((observable, oldValue, newValue) -> {
//            try {
//                applicationConfig.update(ApplicationSettings.TorrentSettings.newBuilder(settings)
//                        .setDownloadRateLimit(torrentSettingService.toSettingsValue(newValue))
//                        .build());
//                showNotification();
//            } catch (NumberFormatException ex) {
//                log.warn("Download rate limit is invalid, {}", ex.getMessage(), ex);
//            }
//        });
    }

    private void initializeUploadLimit() {
        var settings = getSettings();

//        uploadLimit.setTextFormatter(numericTextFormatter());
//        uploadLimit.setValue(torrentSettingService.toDisplayValue(settings.getUploadRateLimit()));
//        uploadLimit.valueProperty().addListener((observable, oldValue, newValue) -> {
//            try {
//                applicationConfig.update(ApplicationSettings.TorrentSettings.newBuilder(settings)
//                        .setUploadRateLimit(Integer.parseInt(newValue))
//                        .build());
//                showNotification();
//            } catch (NumberFormatException ex) {
//                log.warn("Upload rate limit is invalid, {}", ex.getMessage(), ex);
//            }
//        });
    }

    private void initializeConnectionLimit() {
        var settings = getSettings();

//        connectionLimit.setTextFormatter(numericTextFormatter());
//        connectionLimit.setValue(String.valueOf(settings.getConnectionsLimit()));
//        connectionLimit.valueProperty().addListener((observable, oldValue, newValue) -> {
//            try {
//                applicationConfig.update(ApplicationSettings.TorrentSettings.newBuilder(settings)
//                        .setConnectionsLimit(Integer.parseInt(newValue))
//                        .build());
//                showNotification();
//            } catch (NumberFormatException ex) {
//                log.warn("Connection limit is invalid, {}", ex.getMessage(), ex);
//            }
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
                        applicationConfig.update(ApplicationSettings.TorrentSettings.newBuilder(settings)
                                .setDirectory(newValue)
                                .build());
                        cacheChooser.setInitialDirectory(newDirectory);
                        showNotification();
                    }
                });
            } else {
                log.error("Failed to retrieve settings", throwable);
            }
        });
    }

    private void initializeCleaningMode() {
        cleaningMode.setCellFactory(item -> createCleaningModeCell());
        cleaningMode.setButtonCell(createCleaningModeCell());
        cleaningMode.getItems().addAll(ApplicationSettings.TorrentSettings.CleaningMode.values());

        getSettings().whenComplete((settings, throwable) -> {
            if (throwable == null) {
                Platform.runLater(() -> {
                    cleaningMode.getSelectionModel().select(settings.getCleaningMode());
                    cleaningMode.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> onCleaningModeChanged(newValue));
                });
            } else {
                log.error("Failed to retrieve settings", throwable);
            }
        });
    }

    private void onCleaningModeChanged(ApplicationSettings.TorrentSettings.CleaningMode newValue) {
        getSettings().whenComplete((settings, throwable) -> {
            if (throwable == null) {
                applicationConfig.update(ApplicationSettings.TorrentSettings.newBuilder(settings)
                        .setCleaningMode(newValue)
                        .build());
                showNotification();
            } else {
                log.error("Failed to retrieve settings", throwable);
            }
        });
    }

    private CompletableFuture<ApplicationSettings.TorrentSettings> getSettings() {
        return applicationConfig.getSettings().thenApply(ApplicationSettings::getTorrentSettings);
    }

    private ListCell<ApplicationSettings.TorrentSettings.CleaningMode> createCleaningModeCell() {
        return new ListCell<>() {
            @Override
            protected void updateItem(ApplicationSettings.TorrentSettings.CleaningMode item, boolean empty) {
                super.updateItem(item, empty);

                if (!empty) {
                    setText(localeText.get("cleaning_mode_" + item.name().toLowerCase()));
                } else {
                    setText(null);
                }
            }
        };
    }

    @FXML
    private void onCacheDirectoryClicked(MouseEvent event) {
        var node = (Node) event.getSource();
        var scene = node.getScene();
        var window = scene.getWindow();
        var newDirectory = cacheChooser.showDialog(window);

        if (newDirectory != null && newDirectory.isDirectory()) {
            cacheDirectory.setText(newDirectory.getAbsolutePath());
        }
    }
}
