package com.github.yoep.popcorn.view.controllers.tv.sections;

import com.github.yoep.popcorn.activities.ActivityManager;
import javafx.fxml.FXML;
import javafx.scene.input.MouseEvent;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import javax.annotation.PostConstruct;

@Slf4j
@RequiredArgsConstructor
public class PlayerSectionController {
    private final ActivityManager activityManager;

    //region PostConstruct

    @PostConstruct
    private void init() {
        log.trace("Initializing video player component for Spring");
        initializeListeners();
    }

    private void initializeListeners() {

    }

    //Endregion

    @FXML
    private void onPlayerClick(MouseEvent event) {
        event.consume();

    }
}
