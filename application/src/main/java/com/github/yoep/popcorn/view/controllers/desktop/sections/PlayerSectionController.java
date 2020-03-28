package com.github.yoep.popcorn.view.controllers.desktop.sections;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.settings.SettingsService;
import com.github.yoep.popcorn.view.controllers.common.sections.AbstractPlayerSectionController;
import com.github.yoep.popcorn.view.services.VideoPlayerService;
import javafx.animation.PauseTransition;
import javafx.fxml.FXML;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import javafx.util.Duration;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
public class PlayerSectionController extends AbstractPlayerSectionController {
    private boolean uiBlocked;

    //region Constructors

    public PlayerSectionController(ActivityManager activityManager,
                                   SettingsService settingsService,
                                   VideoPlayerService videoPlayerService,
                                   LocaleText localeText) {
        super(activityManager, settingsService, videoPlayerService, localeText);
    }

    //endregion

    //region Initializable

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        super.initialize(url, resourceBundle);
        initializePaneListeners();
    }

    private void initializePaneListeners() {
        playerHeaderPane.setOnMouseEntered(event -> uiBlocked = true);
        playerHeaderPane.setOnMouseExited(event -> uiBlocked = false);

        playerControlsPane.setOnMouseEntered(event -> uiBlocked = true);
        playerControlsPane.setOnMouseExited(event -> uiBlocked = false);
    }

    //endregion

    //region AbstractPlayerSectionController

    @Override
    protected PauseTransition getIdleTimer() {
        return new PauseTransition(Duration.seconds(3));
    }

    @Override
    protected PauseTransition getOffsetTimer() {
        return new PauseTransition(Duration.seconds(2));
    }

    @Override
    protected void onHideOverlay() {
        if (uiBlocked)
            return;

        super.onHideOverlay();
    }

    @Override
    protected void onPlayerKeyPressed(KeyEvent event) {
        super.onPlayerKeyPressed(event);
        switch (event.getCode()) {
            case LEFT:
            case KP_LEFT:
                videoPlayerService.videoTimeOffset(-5000);
                event.consume();
                break;
            case RIGHT:
            case KP_RIGHT:
                videoPlayerService.videoTimeOffset(5000);
                event.consume();
                break;
        }
    }

    //endregion

    //region Functions

    @FXML
    private void onPlayerClick(MouseEvent event) {
        event.consume();
        videoPlayerService.changePlayPauseState();
    }

    //endregion
}
