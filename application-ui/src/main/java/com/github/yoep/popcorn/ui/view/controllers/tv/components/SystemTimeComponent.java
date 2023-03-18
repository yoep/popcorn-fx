package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import javafx.animation.KeyFrame;
import javafx.animation.Timeline;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import javafx.util.Duration;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.time.LocalDateTime;
import java.time.format.DateTimeFormatter;
import java.util.ResourceBundle;

@Slf4j
@RequiredArgsConstructor
public class SystemTimeComponent implements Initializable {
    private static final DateTimeFormatter TIME_FORMAT = DateTimeFormatter.ofPattern("HH:mm");

    final Timeline timer = new Timeline(new KeyFrame(Duration.seconds(15), e -> updateNow()));

    @FXML
    Label time;

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        updateNow();
        timer.setCycleCount(Timeline.INDEFINITE);
        timer.playFromStart();
    }

    private void updateNow() {
        time.setText(LocalDateTime.now().format(TIME_FORMAT));
    }
}
