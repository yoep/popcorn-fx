package com.github.yoep.popcorn.controllers.components;

import com.github.yoep.popcorn.settings.SettingsService;
import com.github.yoep.popcorn.trakt.TraktService;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.task.TaskExecutor;
import org.springframework.stereotype.Component;

import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
@Component
@RequiredArgsConstructor
public class SettingsTraktComponent implements Initializable {
    private final TraktService traktService;
    private final SettingsService settingsService;
    private final TaskExecutor taskExecutor;

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {

    }

    @FXML
    private void onConnectClicked() {
        taskExecutor.execute(traktService::getWatched);
    }
}
