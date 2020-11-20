package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.ui.media.providers.models.Media;
import com.github.yoep.popcorn.ui.settings.SettingsService;
import com.github.yoep.popcorn.ui.view.controllers.common.components.AbstractDetailsComponent;
import com.github.yoep.popcorn.ui.view.controls.Overlay;
import com.github.yoep.popcorn.ui.view.services.HealthService;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.beans.InvalidationListener;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import javafx.scene.control.ListView;
import javafx.scene.layout.Pane;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.lang3.StringUtils;

import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
public abstract class AbstractTvDetailsComponent<T extends Media> extends AbstractDetailsComponent<T> implements Initializable {
    @FXML
    protected Overlay overlay;
    @FXML
    protected Pane qualityButton;
    @FXML
    protected Label qualityButtonLabel;

    protected ListView<String> qualityList;
    protected String quality;

    protected AbstractTvDetailsComponent(LocaleText localeText, ImageService imageService, HealthService healthService, SettingsService settingsService) {
        super(localeText, imageService, healthService, settingsService);
    }

    //region Initializable

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializeQualityList();
    }

    protected void initializeQualityList() {
        qualityList = new ListView<>();

        qualityList.setMaxWidth(100);
        qualityList.getItems().addListener((InvalidationListener) observable -> qualityList.setMaxHeight(50.0 * qualityList.getItems().size()));
        qualityList.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> onQualityChanged(newValue));
    }

    //endregion

    //region Functions

    /**
     * Reset the view details.
     */
    protected void reset() {
        this.quality = null;
    }

    /**
     * Load the health information for the given quality.
     *
     * @param quality The media quality to retrieve the health from.
     */
    protected abstract void loadHealth(String quality);

    private void onQualityChanged(String newValue) {
        if (StringUtils.isEmpty(newValue))
            return;

        quality = newValue;
        qualityButtonLabel.setText(newValue);
        loadHealth(newValue);
    }

    //endregion
}
