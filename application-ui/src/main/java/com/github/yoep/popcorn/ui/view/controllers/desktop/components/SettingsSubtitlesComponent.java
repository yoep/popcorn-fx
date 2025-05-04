package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationSettings;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Subtitle;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.subtitles.SubtitleHelper;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.view.controllers.common.components.AbstractSettingsComponent;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.Node;
import javafx.scene.control.CheckBox;
import javafx.scene.control.ComboBox;
import javafx.scene.control.ListCell;
import javafx.scene.control.TextField;
import javafx.scene.input.MouseEvent;
import javafx.stage.DirectoryChooser;
import lombok.extern.slf4j.Slf4j;

import java.io.File;
import java.net.URL;
import java.util.Arrays;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;

@Slf4j
public class SettingsSubtitlesComponent extends AbstractSettingsComponent implements Initializable {
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

    public SettingsSubtitlesComponent(EventPublisher eventPublisher, LocaleText localeText, ApplicationConfig applicationConfig) {
        super(eventPublisher, localeText, applicationConfig);
    }

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
        defaultSubtitle.setCellFactory(params -> createDefaultSubtitleCell());
        defaultSubtitle.setButtonCell(createDefaultSubtitleCell());
        defaultSubtitle.getItems().addAll(Arrays.stream(Subtitle.Language.values())
                .filter(e -> e != Subtitle.Language.UNRECOGNIZED)
                .toList());

        getSettings().thenAccept(settings -> {
            defaultSubtitle.getSelectionModel().select(settings.getDefaultSubtitle());
            defaultSubtitle.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue)
                    -> onDefaultSubtitleChanged(newValue));
        });
    }

    private void initializeFontFamily() {
        fontFamily.getItems().addAll(Arrays.stream(ApplicationSettings.SubtitleSettings.Family.values())
                .filter(e -> e != ApplicationSettings.SubtitleSettings.Family.UNRECOGNIZED)
                .toList());

        getSettings().thenAccept(settings -> {
            fontFamily.getSelectionModel().select(settings.getFontFamily());
            fontFamily.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue)
                    -> onFontFamilyChanged(newValue));
        });
    }

    private void initializeDecoration() {
        decoration.setCellFactory(param -> createDecorationCell());
        decoration.setButtonCell(createDecorationCell());
        decoration.getItems().addAll(Arrays.stream(ApplicationSettings.SubtitleSettings.DecorationType.values())
                .filter(e -> e != ApplicationSettings.SubtitleSettings.DecorationType.UNRECOGNIZED)
                .toList());

        getSettings().thenAccept(settings -> {
            decoration.getSelectionModel().select(settings.getDecoration());
            decoration.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue)
                    -> onDecorationChanged(newValue));
        });
    }

    private void initializeSize() {
        fontSize.getItems().addAll(SubtitleHelper.supportedFontSizes());

        getSettings().thenAccept(settings -> {
            fontSize.getSelectionModel().select((Integer) settings.getFontSize());
            fontSize.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> onFontSizeChanged(newValue));
        });
    }

    private void initializeBold() {
        getSettings().thenAccept(settings -> {
            fontBold.setSelected(settings.getBold());
            fontBold.selectedProperty().addListener(((observableValue, oldValue, newValue) -> onBoldChanged(newValue)));
        });
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

    private void onDefaultSubtitleChanged(Subtitle.Language newValue) {
        getSettings().thenAccept(settings -> Platform.runLater(() -> {
            applicationConfig.update(ApplicationSettings.SubtitleSettings.newBuilder(settings)
                    .setDefaultSubtitle(newValue)
                    .build());
            showNotification();
        }));
    }

    private void onFontFamilyChanged(ApplicationSettings.SubtitleSettings.Family newValue) {
        getSettings().thenAccept(settings -> Platform.runLater(() -> {
            applicationConfig.update(ApplicationSettings.SubtitleSettings.newBuilder(settings)
                    .setFontFamily(newValue)
                    .build());
            showNotification();
        }));
    }

    private void onDecorationChanged(ApplicationSettings.SubtitleSettings.DecorationType newValue) {
        getSettings().thenAccept(settings -> Platform.runLater(() -> {
            applicationConfig.update(ApplicationSettings.SubtitleSettings.newBuilder(settings)
                    .setDecoration(newValue)
                    .build());
            showNotification();
        }));
    }

    private void onFontSizeChanged(Integer newValue) {
        getSettings().thenAccept(settings -> Platform.runLater(() -> {
            applicationConfig.update(ApplicationSettings.SubtitleSettings.newBuilder(settings)
                    .setFontSize(newValue)
                    .build());
            showNotification();
        }));
    }

    private void onBoldChanged(Boolean newValue) {
        getSettings().thenAccept(settings -> Platform.runLater(() -> {
            applicationConfig.update(ApplicationSettings.SubtitleSettings.newBuilder(settings)
                    .setBold(newValue)
                    .build());
            showNotification();
        }));
    }

    private CompletableFuture<ApplicationSettings.SubtitleSettings> getSettings() {
        return applicationConfig.getSettings().thenApply(ApplicationSettings::getSubtitleSettings);
    }

    private ListCell<ApplicationSettings.SubtitleSettings.DecorationType> createDecorationCell() {
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
    void onCacheDirectoryClicked(MouseEvent event) {
        Node node = (Node) event.getSource();
        File newDirectory = cacheChooser.showDialog(node.getScene().getWindow());

        if (newDirectory != null && newDirectory.isDirectory()) {
            cacheDirectory.setText(newDirectory.getAbsolutePath());
        }
    }

    private static ListCell<Subtitle.Language> createDefaultSubtitleCell() {
        return new ListCell<>() {
            @Override
            protected void updateItem(Subtitle.Language item, boolean empty) {
                super.updateItem(item, empty);
                if (!empty) {
                    setText(SubtitleHelper.getNativeName(item));
                } else {
                    setText(null);
                }
            }
        };
    }
}
