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
    private Pane listSection;
    @FXML
    private Pane detailsSection;

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        detailsSection.setVisible(false);
    }
}
