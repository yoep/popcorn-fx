package com.github.yoep.popcorn.controllers.sections;

import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.DetailsCloseActivity;
import com.github.yoep.popcorn.activities.DetailsShowActivity;
import javafx.application.Platform;
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
    private final ActivityManager activityManager;

    @FXML
    private Pane listSection;
    @FXML
    private Pane detailsSection;

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        detailsSection.setVisible(false);

        activityManager.register(DetailsShowActivity.class, activity -> showDetails());
        activityManager.register(DetailsCloseActivity.class, activity -> Platform.runLater(() -> {
            listSection.setVisible(true);
            detailsSection.setVisible(false);
        }));
    }

    private void showDetails() {
        listSection.setVisible(false);
        detailsSection.setVisible(true);
    }
}
