package com.github.yoep.popcorn.controllers.sections;

import com.github.yoep.popcorn.providers.media.models.Media;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.layout.Pane;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Controller;

import java.net.URL;
import java.util.ResourceBundle;

@Controller
@RequiredArgsConstructor
public class ContentSectionController implements Initializable {
    private final DetailsSectionController detailsController;

    @FXML
    private Pane listSection;
    @FXML
    private Pane detailsSection;

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializeDetailsSection();
    }

    public void showDetails(Media media) {
        listSection.setVisible(false);
        detailsSection.setVisible(true);
        detailsController.load(media);
    }

    private void initializeDetailsSection() {
        detailsSection.setVisible(false);

        detailsController.addListener(() -> {
            listSection.setVisible(true);
            detailsSection.setVisible(false);
        });
    }
}
