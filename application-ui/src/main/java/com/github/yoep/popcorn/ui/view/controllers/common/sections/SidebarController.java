package com.github.yoep.popcorn.ui.view.controllers.common.sections;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.stereotype.ViewController;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.media.filters.model.Category;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.ui.events.CategoryChangedEvent;
import com.github.yoep.popcorn.ui.events.CloseSettingsEvent;
import com.github.yoep.popcorn.ui.events.ShowSettingsEvent;
import javafx.animation.FadeTransition;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.Node;
import javafx.scene.control.Label;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.GridPane;
import javafx.scene.layout.Pane;
import javafx.util.Duration;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
@RequiredArgsConstructor
@ViewController
public class SidebarController implements Initializable {
    static final String ACTIVE_STYLE = "active";

    private final ApplicationConfig applicationConfig;
    private final EventPublisher eventPublisher;

    final FadeTransition slideAnimation = new FadeTransition(Duration.millis(500.0), new Pane());
    Category lastKnownSelectedCategory;

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
    @FXML
    Icon settingsIcon;
    @FXML
    Label settingsText;

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializeSlideAnimation();
        initializeFocusListeners();
        initializeEventListeners();
        initializeActiveIcon();

        sidebar.getColumnConstraints().get(0).setPrefWidth(searchIcon.getPrefWidth());
    }

    private void initializeSlideAnimation() {
        slideAnimation.setFromValue(0);
        slideAnimation.setToValue(1);
        slideAnimation.getNode().setOpacity(0.0);
        slideAnimation.getNode().opacityProperty().addListener((observable, oldValue, newValue) ->
                sidebar.getColumnConstraints().get(1).setMaxWidth(movieText.getPrefWidth() * newValue.doubleValue()));
    }

    private void initializeActiveIcon() {
        var settings = applicationConfig.getSettings().getUiSettings();
        switchCategory(settings.getStartScreen(), true);
    }

    private void initializeFocusListeners() {
        for (Node child : sidebar.getChildren()) {
            child.focusedProperty().addListener((observable, oldValue, newValue) -> focusChanged(newValue));
        }
    }

    private void initializeEventListeners() {
        eventPublisher.register(CloseSettingsEvent.class, event -> {
            switchCategory(lastKnownSelectedCategory, false);
            return event;
        });
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

    private void switchCategory(Category category, boolean publishEvent) {
        Platform.runLater(() -> {
            switch (category) {
                case MOVIES -> switchCategory(movieIcon, publishEvent);
                case SERIES -> switchCategory(serieIcon, publishEvent);
                case FAVORITES -> switchCategory(favoriteIcon, publishEvent);
            }
        });
    }

    private void switchCategory(Icon icon) {
        switchCategory(icon, true);
    }

    private void switchCategory(Icon icon, boolean publishEvent) {
        var category = Category.MOVIES;
        if (icon == serieIcon) {
            category = Category.SERIES;
        }
        if (icon == favoriteIcon) {
            category = Category.FAVORITES;
        }

        switchActiveItem(icon);

        if (publishEvent) {
            log.trace("Category is being changed to \"{}\"", category);
            lastKnownSelectedCategory = category;
            eventPublisher.publish(new CategoryChangedEvent(this, category));
        }
    }

    private void switchActiveItem(Icon icon) {
        var text = movieText;
        movieIcon.getStyleClass().removeIf(e -> e.equals(ACTIVE_STYLE));
        movieText.getStyleClass().removeIf(e -> e.equals(ACTIVE_STYLE));
        serieIcon.getStyleClass().removeIf(e -> e.equals(ACTIVE_STYLE));
        serieText.getStyleClass().removeIf(e -> e.equals(ACTIVE_STYLE));
        favoriteIcon.getStyleClass().removeIf(e -> e.equals(ACTIVE_STYLE));
        favoriteText.getStyleClass().removeIf(e -> e.equals(ACTIVE_STYLE));
        settingsIcon.getStyleClass().removeIf(e -> e.equals(ACTIVE_STYLE));
        settingsText.getStyleClass().removeIf(e -> e.equals(ACTIVE_STYLE));

        if (icon == serieText.getLabelFor()) {
            text = serieText;
        }
        if (icon == favoriteText.getLabelFor()) {
            text = favoriteText;
        }
        if (icon == settingsText.getLabelFor()) {
            text = settingsText;
        }

        icon.getStyleClass().add(ACTIVE_STYLE);
        text.getStyleClass().add(ACTIVE_STYLE);
    }

    private void onSettingsActivated() {
        switchActiveItem(settingsIcon);
        eventPublisher.publish(new ShowSettingsEvent(this));
    }

    @FXML
    void onCategoryClicked(MouseEvent event) {
        if (event.getSource() instanceof Icon icon) {
            event.consume();
            switchCategory(icon);
        } else if (event.getSource() instanceof Label label) {
            event.consume();
            switchCategory((Icon) label.getLabelFor());
        }
    }

    @FXML
    void onCategoryPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            switchCategory((Icon) event.getSource());
        }
    }

    @FXML
    void onSettingsClicked(MouseEvent event) {
        event.consume();
        onSettingsActivated();
    }

    @FXML
    void onSettingsPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            onSettingsActivated();
        }
    }

    @FXML
    void onHovering(MouseEvent event) {
        focusChanged(true);
    }

    @FXML
    void onHoverStopped(MouseEvent event) {
        focusChanged(sidebar.getChildren().stream()
                .anyMatch(Node::isFocused));
    }
}
