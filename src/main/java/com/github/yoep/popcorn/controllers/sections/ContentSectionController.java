package com.github.yoep.popcorn.controllers.sections;

import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.CategoryChangedActivity;
import com.github.yoep.popcorn.activities.CloseDetailsActivity;
import com.github.yoep.popcorn.activities.ShowDetailsActivity;
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

        activityManager.register(ShowDetailsActivity.class, activity -> switchContent(true));
        activityManager.register(CloseDetailsActivity.class, activity -> switchContent(false));
        activityManager.register(CategoryChangedActivity.class, activity -> switchContent(false));
    }

    private void switchContent(boolean isDetailsVisible) {
        listSection.setVisible(!isDetailsVisible);
        detailsSection.setVisible(isDetailsVisible);
    }
}
