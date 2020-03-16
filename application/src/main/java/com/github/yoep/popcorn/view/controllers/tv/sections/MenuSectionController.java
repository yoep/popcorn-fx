package com.github.yoep.popcorn.view.controllers.tv.sections;

import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.CategoryChangedActivity;
import com.github.yoep.popcorn.activities.ShowSettingsActivity;
import com.github.yoep.popcorn.view.models.Category;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.image.Image;
import javafx.scene.image.ImageView;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.Pane;
import javafx.stage.Stage;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.io.ClassPathResource;

import java.io.IOException;
import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
@RequiredArgsConstructor
public class MenuSectionController implements Initializable {
    private static final String ACTIVE_STYLE_CLASS = "active";

    private final ActivityManager activityManager;

    @FXML
    private ImageView header;
    @FXML
    private Pane moviesCategory;
    @FXML
    private Pane seriesCategory;
    @FXML
    private Pane settingsItem;
    @FXML
    private Pane shutdownItem;

    //region Initializable

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeHeader();
        initializeCategory();
    }

    //endregion

    //region Functions

    private void initializeHeader() {
        var headerResource = new ClassPathResource("images/header-small.png");

        try {
            header.setImage(new Image(headerResource.getInputStream()));
        } catch (IOException ex) {
            log.error(ex.getMessage(), ex);
        }
    }

    private void initializeCategory() {
        switchCategory(moviesCategory);
    }

    private void switchCategory(Pane category) {
        moviesCategory.getStyleClass().removeIf(e -> e.equals(ACTIVE_STYLE_CLASS));
        seriesCategory.getStyleClass().removeIf(e -> e.equals(ACTIVE_STYLE_CLASS));

        category.getStyleClass().add(ACTIVE_STYLE_CLASS);

        if (category == moviesCategory) {
            activityManager.register((CategoryChangedActivity) () -> Category.MOVIES);
        }
        if (category == seriesCategory) {
            activityManager.register((CategoryChangedActivity) () -> Category.SERIES);
        }
    }

    private void showSettings() {
        activityManager.register(new ShowSettingsActivity() {
        });
    }

    private void onShutdown() {
        var stage = (Stage) shutdownItem.getScene().getWindow();

        stage.close();
    }

    @FXML
    private void onMoviesCategoryClicked(MouseEvent event) {
        event.consume();
        switchCategory(moviesCategory);
    }

    @FXML
    private void onSeriesCategoryClicked(MouseEvent event) {
        event.consume();
        switchCategory(seriesCategory);
    }

    @FXML
    private void onSettingsClicked(MouseEvent event) {
        event.consume();
        showSettings();
    }

    @FXML
    private void onShutdownClicked(MouseEvent event) {
        event.consume();
        onShutdown();
    }

    @FXML
    private void onKeyEvent(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            var source = event.getSource();

            if (source == moviesCategory || source == seriesCategory) {
                switchCategory((Pane) source);
            }
            if (source == settingsItem) {
                showSettings();
            }
            if (source == shutdownItem) {
                onShutdown();
            }
        }
    }

    //endregion
}
