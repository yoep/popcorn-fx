package com.github.yoep.popcorn.controllers.sections;

import com.github.yoep.popcorn.activities.ActivityManager;
import javafx.fxml.FXML;
import javafx.scene.layout.Pane;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Controller;

import javax.annotation.PostConstruct;

@Slf4j
@Controller
@RequiredArgsConstructor
public class LoaderSectionController {
    private final ActivityManager activityManager;

    @FXML
    private Pane rootPane;

    //region PostConstruct

    @PostConstruct
    private void init() {
        initializeListeners();
    }

    private void initializeListeners() {

    }

    //endregion
}
