package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.settings.SettingsService;
import com.github.yoep.popcorn.backend.settings.models.SubtitleSettings;
import com.github.yoep.popcorn.backend.settings.models.subtitles.DecorationType;
import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleFamily;
import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleLanguage;
import com.github.yoep.popcorn.ui.view.controllers.common.components.AbstractSettingsComponent;
import com.github.yoep.popcorn.ui.view.controllers.tv.sections.SettingsSectionController;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.CheckBox;
import javafx.scene.control.Label;
import javafx.scene.control.ListCell;
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
    private CheckBox fontBold;
    @FXML
    private CheckBox clearCache;
    @FXML
    private Pane defaultSubtitlePane;
    @FXML
    private Pane fontFamilyPane;
    @FXML
    private Pane decorationPane;
    @FXML
    private Pane fontSizePane;

    private ListView<SubtitleLanguage> defaultSubtitleList;
    private ListView<SubtitleFamily> fontFamilyList;
    private ListView<DecorationType> decorationList;
    private ListView<Integer> fontSizeList;

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
        initializeFontBold();
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
        makeListViewHeightAdaptive(defaultSubtitleList);
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

        fontFamilyList = new ListView<>();
        makeListViewHeightAdaptive(fontFamilyList);
        fontFamilyList.getItems().addAll(SubtitleFamily.values());
        fontFamilyList.getSelectionModel().select(settings.getFontFamily());
        fontFamilyList.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> {
            var subtitleSettings = getSubtitleSettings();
            subtitleSettings.setFontFamily(newValue);
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

        decorationList = new ListView<>();
        makeListViewHeightAdaptive(decorationList);
        decorationList.setCellFactory(param -> createDecorationListCell());
        decorationList.getItems().addAll(DecorationType.values());
        decorationList.getSelectionModel().select(settings.getDecoration());
        decorationList.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> {
            var subtitleSettings = getSubtitleSettings();
            subtitleSettings.setDecoration(newValue);
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

        fontSizeList = new ListView<>();
        makeListViewHeightAdaptive(fontSizeList);
        fontSizeList.getItems().addAll(SubtitleSettings.supportedFontSizes());
        fontSizeList.getSelectionModel().select((Integer) settings.getFontSize());
        fontSizeList.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> {
            var subtitleSettings = getSubtitleSettings();
            subtitleSettings.setFontSize(newValue);
        });
    }

    private void initializeFontBold() {
        var settings = getSubtitleSettings();

        updateFontBold(settings.isBold());
        settings.addListener(evt -> {
            if (evt.getPropertyName().equals(SubtitleSettings.BOLD_PROPERTY)) {
                updateFontBold((Boolean) evt.getNewValue());
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

    private void updateFontBold(boolean enabled) {
        this.fontBold.setSelected(enabled);
    }

    private void updateClearCache(boolean enabled) {
        this.clearCache.setSelected(enabled);
    }

    private void onDefaultSubtitleEvent() {
        settingsSection.showOverlay(defaultSubtitlePane, defaultSubtitleList);
    }

    private void onFontFamilyEvent() {
        settingsSection.showOverlay(fontFamilyPane, fontFamilyList);
    }

    private void onDecorationEvent() {
        settingsSection.showOverlay(decorationPane, decorationList);
    }

    private void onFontSizeEvent() {
        settingsSection.showOverlay(fontSizePane, fontSizeList);
    }

    private void onFontBoldEvent() {
        var settings = getSubtitleSettings();
        settings.setBold(fontBold.isSelected());
    }

    private void onClearCacheEvent() {
        var settings = getSubtitleSettings();
        settings.setAutoCleaningEnabled(clearCache.isSelected());
    }

    private SubtitleSettings getSubtitleSettings() {
        return settingsService.getSettings().getSubtitleSettings();
    }

    private ListCell<DecorationType> createDecorationListCell() {
        return new ListCell<>() {
            @Override
            protected void updateItem(DecorationType item, boolean empty) {
                super.updateItem(item, empty);

                if (empty) {
                    setText(null);
                } else {
                    setText(localeText.get("settings_subtitles_style_" + item.toString().toLowerCase()));
                }
            }
        };
    }

    @FXML
    private void onDefaultSubtitleKeyPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            onDefaultSubtitleEvent();
        }
    }

    @FXML
    private void onFontFamilyKeyPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            onFontFamilyEvent();
        }
    }

    @FXML
    private void onDecorationKeyPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            onDecorationEvent();
        }
    }

    @FXML
    private void onFontSizeKeyPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            onFontSizeEvent();
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
    private void onFontBoldKeyPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            onFontBoldEvent();
        }
    }

    @FXML
    private void onDefaultSubtitleClicked(MouseEvent event) {
        event.consume();
        onDefaultSubtitleEvent();
    }

    @FXML
    private void onFontFamilyClicked(MouseEvent event) {
        event.consume();
        onFontFamilyEvent();
    }

    @FXML
    private void onDecorationClicked(MouseEvent event) {
        event.consume();
        onDecorationEvent();
    }

    @FXML
    private void onFontSizeClicked(MouseEvent event) {
        event.consume();
        onFontSizeEvent();
    }

    @FXML
    private void onClearCacheClicked(MouseEvent event) {
        event.consume();
        onClearCacheEvent();
    }

    @FXML
    private void onFontBoldClicked(MouseEvent event) {
        event.consume();
        onFontBoldEvent();
    }

    //endregion
}
