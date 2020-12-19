package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.ui.settings.SettingsService;
import com.github.yoep.popcorn.ui.settings.models.TorrentSettings;
import com.github.yoep.popcorn.ui.view.controllers.common.components.AbstractSettingsUiComponent;
import com.github.yoep.popcorn.ui.view.controllers.tv.sections.SettingsSectionController;
import com.github.yoep.popcorn.ui.view.services.TorrentSettingService;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.CheckBox;
import javafx.scene.control.Label;
import javafx.scene.control.TextField;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.Pane;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.ApplicationEventPublisher;

import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
public class SettingsTorrentComponent extends AbstractSettingsUiComponent implements Initializable {
    private final TorrentSettingService torrentSettingService;
    private final SettingsSectionController settingsSection;

    @FXML
    private Pane downloadLimitPane;
    @FXML
    private Label downloadLimit;
    @FXML
    private Pane uploadLimitPane;
    @FXML
    private Label uploadLimit;
    @FXML
    private Pane connectionLimitPane;
    @FXML
    private Label connectionLimit;
    @FXML
    private CheckBox clearCache;

    private TextField downloadLimitInput;
    private TextField uploadLimitInput;
    private TextField connectionLimitInput;

    public SettingsTorrentComponent(ApplicationEventPublisher eventPublisher,
                                    LocaleText localeText,
                                    SettingsService settingsService,
                                    TorrentSettingService torrentSettingService, SettingsSectionController settingsSection) {
        super(eventPublisher, localeText, settingsService);
        this.torrentSettingService = torrentSettingService;
        this.settingsSection = settingsSection;
    }

    //region Initializable

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeDownloadLimit();
        initializeUploadLimit();
        initializeConnectionLimit();
        initializeClearCache();
    }

    private void initializeDownloadLimit() {
        var settings = getSettings();

        downloadLimitInput = new TextField();
        downloadLimitInput.setOnKeyPressed(event -> {
            if (event.getCode() == KeyCode.ENTER) {
                onDownloadLimitChanged(downloadLimitInput.getText());
            }
        });
        settings.addListener(event -> {
            if (event.getPropertyName().equals(TorrentSettings.DOWNLOAD_RATE_PROPERTY)) {
                updateDownloadLimit();
            }
        });

        updateDownloadLimit();
    }

    private void initializeUploadLimit() {
        var settings = getSettings();

        uploadLimitInput = new TextField();
        uploadLimitInput.setOnKeyPressed(event -> {
            if (event.getCode() == KeyCode.ENTER) {
                onUploadLimitChanged(uploadLimitInput.getText());
            }
        });
        settings.addListener(event -> {
            if (event.getPropertyName().equals(TorrentSettings.UPLOAD_RATE_PROPERTY)) {
                updateUploadLimit();
            }
        });

        updateUploadLimit();
    }

    private void initializeConnectionLimit() {
        var settings = getSettings();

        connectionLimitInput = new TextField();
        connectionLimitInput.setOnKeyPressed(event -> {
            if (event.getCode() == KeyCode.ENTER) {
                onConnectionLimitChanged(connectionLimitInput.getText());
            }
        });
        settings.addListener(event -> {
            if (event.getPropertyName().equals(TorrentSettings.CONNECTIONS_LIMIT_PROPERTY)) {
                updateConnectionLimit();
            }
        });

        updateConnectionLimit();
    }

    private void initializeClearCache() {
        var settings = getSettings();

        clearCache.setSelected(settings.isAutoCleaningEnabled());
        clearCache.selectedProperty().addListener((observable, oldValue, newValue) -> settings.setAutoCleaningEnabled(newValue));
    }

    //endregion

    //region Functions

    private void updateDownloadLimit() {
        var settings = getSettings();
        var value = torrentSettingService.toDisplayValue(settings.getDownloadRateLimit());

        downloadLimit.setText(value);
        downloadLimitInput.setText(value);
    }

    private void updateUploadLimit() {
        var settings = getSettings();
        var value = torrentSettingService.toDisplayValue(settings.getUploadRateLimit());

        uploadLimit.setText(value);
        uploadLimitInput.setText(value);
    }

    private void updateConnectionLimit() {
        var settings = getSettings();
        var value = String.valueOf(settings.getConnectionsLimit());

        connectionLimit.setText(value);
        connectionLimitInput.setText(value);
    }

    private void onDownloadLimitChanged(String newValue) {
        var settings = getSettings();

        try {
            settings.setDownloadRateLimit(torrentSettingService.toSettingsValue(newValue));
            showNotification();
        } catch (NumberFormatException ex) {
            log.warn("Download rate limit is invalid, " + ex.getMessage(), ex);
        }
    }

    private void onUploadLimitChanged(String newValue) {
        var settings = getSettings();

        try {
            settings.setUploadRateLimit(torrentSettingService.toSettingsValue(newValue));
            showNotification();
        } catch (NumberFormatException ex) {
            log.warn("Upload rate limit is invalid, " + ex.getMessage(), ex);
        }
    }

    private void onConnectionLimitChanged(String newValue) {
        var settings = getSettings();

        try {
            settings.setConnectionsLimit(Integer.parseInt(newValue));
            showNotification();
        } catch (NumberFormatException ex) {
            log.warn("Connection limit is invalid, " + ex.getMessage(), ex);
        }
    }

    private void onDownloadLimitEvent() {
        settingsSection.setBackspaceActionEnabled(false);
        settingsSection.showOverlay(downloadLimitPane, downloadLimitInput);
    }

    private void onUploadLimitEvent() {
        settingsSection.setBackspaceActionEnabled(false);
        settingsSection.showOverlay(uploadLimitPane, uploadLimitInput);
    }

    private void onConnectionLimitEvent() {
        settingsSection.setBackspaceActionEnabled(false);
        settingsSection.showOverlay(connectionLimitPane, connectionLimitInput);
    }

    private TorrentSettings getSettings() {
        return settingsService.getSettings().getTorrentSettings();
    }

    @FXML
    private void onDownloadLimitKeyPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            onDownloadLimitEvent();
        }
    }

    @FXML
    private void onDownloadLimitClicked(MouseEvent event) {
        event.consume();
        onDownloadLimitEvent();
    }

    @FXML
    private void onUploadLimitKeyPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            onUploadLimitEvent();
        }
    }

    @FXML
    private void onUploadLimitClicked(MouseEvent event) {
        event.consume();
        onUploadLimitEvent();
    }

    @FXML
    private void onConnectionLimitKeyPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            onConnectionLimitEvent();
        }
    }

    @FXML
    private void onConnectionLimitClicked(MouseEvent event) {
        event.consume();
        onConnectionLimitEvent();
    }

    //endregion
}
