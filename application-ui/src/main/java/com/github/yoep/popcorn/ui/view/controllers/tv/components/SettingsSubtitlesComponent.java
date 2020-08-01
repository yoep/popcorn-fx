package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.ui.activities.ActivityManager;
import com.github.yoep.popcorn.ui.settings.SettingsService;
import com.github.yoep.popcorn.ui.settings.models.SubtitleSettings;
import com.github.yoep.popcorn.ui.subtitles.models.DecorationType;
import com.github.yoep.popcorn.ui.subtitles.models.SubtitleFamily;
import com.github.yoep.popcorn.ui.subtitles.models.SubtitleLanguage;
import com.github.yoep.popcorn.ui.view.controllers.common.components.AbstractSettingsComponent;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;

import java.net.URL;
import java.util.ResourceBundle;

public class SettingsSubtitlesComponent extends AbstractSettingsComponent implements Initializable {

    @FXML
    private Label defaultSubtitle;
    @FXML
    private Label fontFamily;
    @FXML
    private Label decoration;
    @FXML
    private Label fontSize;

    //region Constructors

    public SettingsSubtitlesComponent(ActivityManager activityManager, LocaleText localeText, SettingsService settingsService) {
        super(activityManager, localeText, settingsService);
    }

    //endregion

    //region Initializable

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializeDefaultSubtitle();
        initializeFontFamily();
        initializeDecoration();
        initializeFontSize();
    }

    private void initializeDefaultSubtitle() {
        var settings = getSubtitleSettings();

        updateDefaultSubtitle(settings.getDefaultSubtitle());
        settings.addListener(evt -> {
            if (evt.getPropertyName().equals(SubtitleSettings.DEFAULT_SUBTITLE_PROPERTY)) {
                updateDefaultSubtitle((SubtitleLanguage) evt.getNewValue());
            }
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

    private SubtitleSettings getSubtitleSettings() {
        return settingsService.getSettings().getSubtitleSettings();
    }

    //endregion
}
