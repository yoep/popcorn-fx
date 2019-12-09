package com.github.yoep.popcorn.controllers.sections;

import com.github.yoep.popcorn.activities.ActivityListener;
import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.DetailsCloseActivity;
import com.github.yoep.popcorn.activities.DetailsShowActivity;
import com.github.yoep.popcorn.media.providers.models.Media;
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
        initializeDetailsSection();
    }

    public void showDetails(Media media) {
        listSection.setVisible(false);
        detailsSection.setVisible(true);

        activityManager.register((DetailsShowActivity) () -> media);
    }

    private void initializeDetailsSection() {
        detailsSection.setVisible(false);

        activityManager.register(DetailsCloseActivity.class, activity -> Platform.runLater(() -> {
            listSection.setVisible(true);
            detailsSection.setVisible(false);
        }));
    }
}
