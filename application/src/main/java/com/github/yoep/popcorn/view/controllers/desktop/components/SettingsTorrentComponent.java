package com.github.yoep.popcorn.view.controllers.desktop.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.settings.SettingsService;
import com.github.yoep.popcorn.settings.models.TorrentSettings;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.Node;
import javafx.scene.control.CheckBox;
import javafx.scene.control.TextField;
import javafx.scene.input.MouseEvent;
import javafx.stage.DirectoryChooser;
import lombok.extern.slf4j.Slf4j;

import java.io.File;
import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
public class SettingsTorrentComponent extends AbstractSettingsComponent implements Initializable {
    private final DirectoryChooser cacheChooser = new DirectoryChooser();

    @FXML
    private TextField downloadLimit;
    @FXML
    private TextField uploadLimit;
    @FXML
    private TextField connectionLimit;
    @FXML
    private TextField cacheDirectory;
    @FXML
    private CheckBox clearCache;

    public SettingsTorrentComponent(ActivityManager activityManager, LocaleText localeText, SettingsService settingsService) {
        super(activityManager, localeText, settingsService);
    }

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeDownloadLimit();
        initializeUploadLimit();
        initializeConnectionLimit();
        initializeCacheDirectory();
        initializeClearCache();
    }

    private void initializeDownloadLimit() {
        var settings = getSettings();

        downloadLimit.setText(String.valueOf(settings.getDownloadRateLimit()));
        downloadLimit.textProperty().addListener((observable, oldValue, newValue) -> {
            try {
                settings.setDownloadRateLimit(Integer.parseInt(newValue));
                showNotification();
            } catch (NumberFormatException ex) {
                log.warn("Download rate limit is invalid, " + ex.getMessage(), ex);
            }
        });
    }

    private void initializeUploadLimit() {
        var settings = getSettings();

        uploadLimit.setText(String.valueOf(settings.getUploadRateLimit()));
        uploadLimit.textProperty().addListener((observable, oldValue, newValue) -> {
            try {
                settings.setUploadRateLimit(Integer.parseInt(newValue));
                showNotification();
            } catch (NumberFormatException ex) {
                log.warn("Upload rate limit is invalid, " + ex.getMessage(), ex);
            }
        });
    }

    private void initializeConnectionLimit() {
        var settings = getSettings();

        connectionLimit.setText(String.valueOf(settings.getConnectionsLimit()));
        connectionLimit.textProperty().addListener((observable, oldValue, newValue) -> {
            try {
                settings.setConnectionsLimit(Integer.parseInt(newValue));
                showNotification();
            } catch (NumberFormatException ex) {
                log.warn("Connection limit is invalid, " + ex.getMessage(), ex);
            }
        });
    }

    private void initializeCacheDirectory() {
        var settings = getSettings();
        var directory = settings.getDirectory();

        cacheChooser.setInitialDirectory(directory);
        cacheDirectory.setText(directory.getAbsolutePath());
        cacheDirectory.textProperty().addListener((observable, oldValue, newValue) -> {
            File newDirectory = new File(newValue);

            if (newDirectory.isDirectory()) {
                settings.setDirectory(newDirectory);
                cacheChooser.setInitialDirectory(newDirectory);
                showNotification();
            }
        });
    }

    private void initializeClearCache() {
        var settings = getSettings();

        clearCache.setSelected(settings.isAutoCleaningEnabled());
        clearCache.selectedProperty().addListener((observable, oldValue, newValue) -> onClearCacheChanged(newValue));
    }

    private void onClearCacheChanged(Boolean newValue) {
        var settings = getSettings();

        settings.setAutoCleaningEnabled(newValue);
        showNotification();
    }

    private TorrentSettings getSettings() {
        return settingsService.getSettings().getTorrentSettings();
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
