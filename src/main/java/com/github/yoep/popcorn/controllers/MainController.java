package com.github.yoep.popcorn.controllers;

import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.layout.Pane;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Controller;

import java.net.URL;
import java.util.ResourceBundle;

@Controller
@RequiredArgsConstructor
public class MainController implements Initializable {
    @FXML
    private Pane contentSection;
    @FXML
    private Pane playerSection;

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        switchSection(false);
    }

    private void switchSection(boolean showPlayer) {
        contentSection.setVisible(!showPlayer);
        playerSection.setVisible(showPlayer);
    }
}
