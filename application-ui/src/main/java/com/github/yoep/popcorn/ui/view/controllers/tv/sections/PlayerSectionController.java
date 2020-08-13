package com.github.yoep.popcorn.ui.view.controllers.tv.sections;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.ui.events.ActivityManager;
import com.github.yoep.popcorn.ui.settings.SettingsService;
import com.github.yoep.popcorn.ui.view.controllers.common.sections.AbstractPlayerSectionController;
import com.github.yoep.popcorn.ui.view.services.VideoPlayerService;
import javafx.animation.PauseTransition;
import javafx.fxml.FXML;
import javafx.scene.input.MouseEvent;
import javafx.util.Duration;
import lombok.extern.slf4j.Slf4j;

@Slf4j
public class PlayerSectionController extends AbstractPlayerSectionController {
    //region Constructors

    public PlayerSectionController(ActivityManager activityManager,
                                   SettingsService settingsService,
                                   VideoPlayerService videoPlayerService,
                                   LocaleText localeText) {
        super(activityManager, settingsService, videoPlayerService, localeText);
    }

    //endregion

    //region AbstractPlayerSectionController

    @Override
    protected PauseTransition getIdleTimer() {
        return new PauseTransition(Duration.seconds(5));
    }

    @Override
    protected PauseTransition getOffsetTimer() {
        return new PauseTransition(Duration.seconds(3));
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
