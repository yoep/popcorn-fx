package com.github.yoep.popcorn.ui.view.controllers.common.sections;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.stereotype.ViewController;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.media.filters.model.Category;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.ui.events.*;
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
    private final ViewLoader viewLoader;

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
    @FXML
    Icon infoIcon;
    @FXML
    Label infoText;

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializeSlideAnimation();
        initializeEventListeners();
        initializeActiveIcon();
        initializeSearch();
        initializeFocusListeners();

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
            child.focusWithinProperty().addListener((observable, oldValue, newValue) -> focusChanged(newValue));
        }
    }

    private void initializeEventListeners() {
        eventPublisher.register(CloseSettingsEvent.class, event -> {
            switchCategory(lastKnownSelectedCategory, false);
            return event;
        });
        eventPublisher.register(CloseAboutEvent.class, event -> {
            switchCategory(lastKnownSelectedCategory, false);
            return event;
        });
    }

    private void initializeSearch() {
        var search = viewLoader.load("components/sidebar-search.component.fxml");

        GridPane.setColumnIndex(search, 1);
        GridPane.setRowIndex(search, 1);
        sidebar.getChildren().add(2, search);
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

        for (var i = 0; i < sidebar.getChildren().size(); i++) {
            var child = sidebar.getChildren().get(i);
            child.getStyleClass().removeIf(e -> e.equals(ACTIVE_STYLE));
        }

        if (icon == serieText.getLabelFor()) {
            text = serieText;
        }
        if (icon == favoriteText.getLabelFor()) {
            text = favoriteText;
        }
        if (icon == settingsText.getLabelFor()) {
            text = settingsText;
        }
        if (icon == infoText.getLabelFor()) {
            text = infoText;
        }

        icon.getStyleClass().add(ACTIVE_STYLE);
        text.getStyleClass().add(ACTIVE_STYLE);
    }

    private void onSettingsActivated() {
        switchActiveItem(settingsIcon);
        eventPublisher.publish(new ShowSettingsEvent(this));
    }

    private void onInfoActivated() {
        switchActiveItem(infoIcon);
        eventPublisher.publish(new ShowAboutEvent(this));
    }

    private void onSearchFocusRequest() {
        eventPublisher.publish(new RequestSearchFocus(this));
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
    void onInfoClicked(MouseEvent event) {
        event.consume();
        onInfoActivated();
    }

    @FXML
    void onInfoPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            onInfoActivated();
        }
    }

    @FXML
    void onHovering(MouseEvent event) {
        focusChanged(true);
    }

    @FXML
    void onHoverStopped(MouseEvent event) {
        focusChanged(sidebar.getChildren().stream()
                .anyMatch(Node::isFocusWithin));
    }

    @FXML
    void onSearchClicked(MouseEvent event) {
        event.consume();
        onSearchFocusRequest();
    }

    @FXML
    void onSearchPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            onSearchFocusRequest();
        }
    }
}
