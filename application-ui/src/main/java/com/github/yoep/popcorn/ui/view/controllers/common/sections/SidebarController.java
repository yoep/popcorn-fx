package com.github.yoep.popcorn.ui.view.controllers.common.sections;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.stereotype.ViewController;
import com.github.yoep.popcorn.backend.media.filters.model.Category;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.ui.events.CategoryChangedEvent;
import javafx.animation.FadeTransition;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.GridPane;
import javafx.scene.layout.Pane;
import javafx.util.Duration;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.ApplicationEventPublisher;

import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
@RequiredArgsConstructor
@ViewController
public class SidebarController implements Initializable {
    static final String ACTIVE_STYLE = "active";

    private final ApplicationConfig applicationConfig;
    private final ApplicationEventPublisher eventPublisher;

    private final FadeTransition slideAnimation = new FadeTransition(Duration.seconds(1.0), new Pane());

    @FXML
    GridPane sidebar;
    @FXML
    Icon searchIcon;
    @FXML
    Icon movieIcon;
    @FXML
    Label movieText;
    @FXML
    Icon serieIcon;
    @FXML
    Label serieText;
    @FXML
    Icon favoriteIcon;
    @FXML
    Label favoriteText;

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializeActiveIcon();
        initializeIconFocusListeners();

        slideAnimation.setFromValue(0);
        slideAnimation.setToValue(1);
        slideAnimation.getNode().opacityProperty().addListener((observable, oldValue, newValue) ->
                sidebar.getColumnConstraints().get(1).setMaxWidth(125 * newValue.doubleValue()));
    }

    private void initializeActiveIcon() {
        var settings = applicationConfig.getSettings().getUiSettings();
        Platform.runLater(() -> {
            switch (settings.getStartScreen()) {
                case MOVIES -> switchActive(movieIcon);
                case SERIES -> switchActive(serieIcon);
                case FAVORITES -> switchActive(favoriteIcon);
            }
        });
    }

    private void initializeIconFocusListeners() {
        searchIcon.focusedProperty().addListener((observable, oldValue, newValue) -> focusChanged(newValue));
        movieIcon.focusedProperty().addListener((observable, oldValue, newValue) -> focusChanged(newValue));
        serieIcon.focusedProperty().addListener((observable, oldValue, newValue) -> focusChanged(newValue));
        favoriteIcon.focusedProperty().addListener((observable, oldValue, newValue) -> focusChanged(newValue));
    }

    private void focusChanged(boolean newValue) {
        if (newValue) {
            slideAnimation.setFromValue(slideAnimation.getNode().getOpacity());
            slideAnimation.setToValue(1);
            slideAnimation.playFromStart();
        } else {
            slideAnimation.setFromValue(slideAnimation.getNode().getOpacity());
            slideAnimation.setToValue(0);
            slideAnimation.playFromStart();
        }
    }

    private void switchActive(Icon icon) {
        var category = Category.MOVIES;
        var text = movieText;
        movieIcon.getStyleClass().removeIf(e -> e.equals(ACTIVE_STYLE));
        movieText.getStyleClass().removeIf(e -> e.equals(ACTIVE_STYLE));
        serieIcon.getStyleClass().removeIf(e -> e.equals(ACTIVE_STYLE));
        serieText.getStyleClass().removeIf(e -> e.equals(ACTIVE_STYLE));
        favoriteIcon.getStyleClass().removeIf(e -> e.equals(ACTIVE_STYLE));
        favoriteText.getStyleClass().removeIf(e -> e.equals(ACTIVE_STYLE));

        if (icon == serieText.getLabelFor()) {
            text = serieText;
        }
        if (icon == favoriteText.getLabelFor()) {
            text = favoriteText;
        }

        icon.getStyleClass().add(ACTIVE_STYLE);
        text.getStyleClass().add(ACTIVE_STYLE);

        if (icon == serieIcon) {
            category = Category.SERIES;
        }
        if (icon == favoriteIcon) {
            category = Category.FAVORITES;
        }

        log.trace("Category is being changed to \"{}\"", category);
        eventPublisher.publishEvent(new CategoryChangedEvent(this, category));
    }

    @FXML
    void onCategoryClicked(MouseEvent event) {
        event.consume();
        switchActive((Icon) event.getSource());
    }

    @FXML
    void onCategoryPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            switchActive((Icon) event.getSource());
        }
    }

    @FXML
    void onHovering(MouseEvent event) {
        focusChanged(true);
    }

    @FXML
    void onHoverStopped(MouseEvent event) {
        focusChanged(false);
    }
}
