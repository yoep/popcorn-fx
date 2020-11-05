package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.ui.settings.SettingsService;
import com.github.yoep.popcorn.ui.settings.models.SubtitleSettings;
import com.github.yoep.popcorn.ui.subtitles.models.DecorationType;
import com.github.yoep.popcorn.ui.subtitles.models.SubtitleFamily;
import com.github.yoep.popcorn.ui.subtitles.models.SubtitleLanguage;
import com.github.yoep.popcorn.ui.view.controllers.common.components.AbstractSettingsComponent;
import com.github.yoep.popcorn.ui.view.controllers.tv.sections.SettingsSectionController;
import javafx.beans.InvalidationListener;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.CheckBox;
import javafx.scene.control.Label;
import javafx.scene.control.ListView;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.Pane;
import org.springframework.context.ApplicationEventPublisher;

import java.net.URL;
import java.util.ResourceBundle;

public class SettingsSubtitlesComponent extends AbstractSettingsComponent implements Initializable {
    private final SettingsSectionController settingsSection;

    @FXML
    private Label defaultSubtitle;
    @FXML
    private Label fontFamily;
    @FXML
    private Label decoration;
    @FXML
    private Label fontSize;
    @FXML
    private CheckBox clearCache;
    @FXML
    private Pane defaultSubtitlePane;

    private ListView<SubtitleLanguage> defaultSubtitleList;

    //region Constructors

    public SettingsSubtitlesComponent(ApplicationEventPublisher eventPublisher, LocaleText localeText, SettingsService settingsService,
                                      SettingsSectionController settingsSection) {
        super(eventPublisher, localeText, settingsService);
        this.settingsSection = settingsSection;
    }

    //endregion

    //region Initializable

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializeDefaultSubtitle();
        initializeFontFamily();
        initializeDecoration();
        initializeFontSize();
        initializeClearCache();
    }

    private void initializeDefaultSubtitle() {
        var settings = getSubtitleSettings();

        updateDefaultSubtitle(settings.getDefaultSubtitle());
        settings.addListener(evt -> {
            if (evt.getPropertyName().equals(SubtitleSettings.DEFAULT_SUBTITLE_PROPERTY)) {
                updateDefaultSubtitle((SubtitleLanguage) evt.getNewValue());
            }
        });

        defaultSubtitleList = new ListView<>();
        defaultSubtitleList.getItems().addListener((InvalidationListener) observable -> defaultSubtitleList.setMaxHeight(50.0 * defaultSubtitleList.getItems().size()));
        defaultSubtitleList.getItems().addAll(SubtitleLanguage.values());
        defaultSubtitleList.getSelectionModel().select(settings.getDefaultSubtitle());
        defaultSubtitleList.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> {
            var subtitleSettings = getSubtitleSettings();
            subtitleSettings.setDefaultSubtitle(newValue);
        });
    }

    private void initializeFontFamily() {
        var settings = getSubtitleSettings();

        updateFontFamily(settings.getFontFamily());
        settings.addListener(evt -> {
            if (evt.getPropertyName().equals(SubtitleSettings.FONT_FAMILY_PROPERTY)) {
                updateFontFamily((SubtitleFamily) evt.getNewValue());
            }
        });
    }

    private void initializeDecoration() {
        var settings = getSubtitleSettings();

        updateDecoration(settings.getDecoration());
        settings.addListener(evt -> {
            if (evt.getPropertyName().equals(SubtitleSettings.DECORATION_PROPERTY)) {
                updateDecoration((DecorationType) evt.getNewValue());
            }
        });
    }

    private void initializeFontSize() {
        var settings = getSubtitleSettings();

        updateFontSize(settings.getFontSize());
        settings.addListener(evt -> {
            if (evt.getPropertyName().equals(SubtitleSettings.FONT_SIZE_PROPERTY)) {
                updateFontSize((Integer) evt.getNewValue());
            }
        });
    }

    private void initializeClearCache() {
        var settings = getSubtitleSettings();

        updateClearCache(settings.isAutoCleaningEnabled());
        settings.addListener(evt -> {
            if (evt.getPropertyName().equals(SubtitleSettings.AUTO_CLEANING_PROPERTY)) {
                updateClearCache((Boolean) evt.getNewValue());
            }
        });
    }

    //endregion

    //region Functions

    private void updateDefaultSubtitle(SubtitleLanguage language) {
        defaultSubtitle.setText(language.toString());
    }

    private void updateFontFamily(SubtitleFamily fontFamily) {
        this.fontFamily.setText(fontFamily.getFamily());
    }

    private void updateDecoration(DecorationType decoration) {
        this.decoration.setText(localeText.get("settings_subtitles_style_" + decoration.toString().toLowerCase()));
    }

    private void updateFontSize(int fontSize) {
        this.fontSize.setText(String.valueOf(fontSize));
    }

    private void updateClearCache(boolean enabled) {
        this.clearCache.setSelected(enabled);
    }

    private void onDefaultSubtitleEvent() {
        settingsSection.showOverlay(defaultSubtitlePane, defaultSubtitleList);
    }

    private void onClearCacheEvent() {
        var settings = getSubtitleSettings();
        settings.setAutoCleaningEnabled(clearCache.isSelected());
    }

    private SubtitleSettings getSubtitleSettings() {
        return settingsService.getSettings().getSubtitleSettings();
    }

    @FXML
    private void onDefaultSubtitleKeyPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            onDefaultSubtitleEvent();
        }
    }

    @FXML
    private void onClearCacheKeyPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            onClearCacheEvent();
        }
    }

    @FXML
    private void onDefaultSubtitleClicked(MouseEvent event) {
        event.consume();
        onDefaultSubtitleEvent();
    }

    //endregion
}
