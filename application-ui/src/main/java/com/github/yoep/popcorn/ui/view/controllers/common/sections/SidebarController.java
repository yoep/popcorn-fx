package com.github.yoep.popcorn.ui.view.controllers.common.sections;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.stereotype.ViewController;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.media.filters.model.Category;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.updater.UpdateCallbackEvent;
import com.github.yoep.popcorn.backend.updater.UpdateService;
import com.github.yoep.popcorn.backend.updater.UpdateState;
import com.github.yoep.popcorn.ui.events.*;
import javafx.animation.Animation;
import javafx.animation.FadeTransition;
import javafx.animation.Transition;
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
import javafx.scene.paint.Color;
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
    private final UpdateService updateService;

    final FadeTransition slideAnimation = new FadeTransition(Duration.millis(500), new Pane());
    final Transition updateTransition = createColorTransition();
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
    Icon collectionIcon;
    @FXML
    Label collectionText;
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
        initializeAnimations();
        initializeEventListeners();
        activateStartCategory();
        initializeSearch();
        initializeFocusListeners();
        initializeMode();

        sidebar.getColumnConstraints().get(0).setPrefWidth(searchIcon.getPrefWidth());
    }

    private void initializeAnimations() {
        slideAnimation.setFromValue(0);
        slideAnimation.setToValue(1);
        slideAnimation.getNode().setOpacity(0.0);
        slideAnimation.getNode().opacityProperty().addListener((observable, oldValue, newValue) ->
                sidebar.getColumnConstraints().get(1).setMaxWidth(movieText.getPrefWidth() * newValue.doubleValue()));
    }

    private void initializeFocusListeners() {
        sidebar.focusWithinProperty().addListener((observable, oldValue, newValue) -> focusChanged(newValue));
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
        eventPublisher.register(CloseUpdateEvent.class, event -> {
            switchCategory(lastKnownSelectedCategory, false);
            return event;
        });
        eventPublisher.register(HomeEvent.class, event -> {
            activateStartCategory();
            return event;
        });
        eventPublisher.register(ContextMenuEvent.class, event -> {
            Platform.runLater(() -> {
                switch (lastKnownSelectedCategory) {
                    case MOVIES -> movieIcon.requestFocus();
                    case SERIES -> serieIcon.requestFocus();
                    case FAVORITES -> favoriteIcon.requestFocus();
                }
            });
            return event;
        });
        updateService.register(event -> {
            if (event.getTag() == UpdateCallbackEvent.Tag.StateChanged) {
                onUpdateStateChanged(event.getUnion().getState_changed().getNewState());
            }
        });
        onUpdateStateChanged(updateService.getState());
    }

    private void initializeMode() {
        if (applicationConfig.isTvMode()) {
            log.trace("Removing torrent collection from sidebar");
            sidebar.getChildren().remove(collectionIcon);
            sidebar.getChildren().remove(collectionText);
        }
    }

    private void onUpdateStateChanged(UpdateState newState) {
        Platform.runLater(() -> {
            if (newState == UpdateState.UPDATE_AVAILABLE) {
                updateTransition.playFromStart();
            } else {
                updateTransition.stop();
            }
        });
    }

    private void initializeSearch() {
        var search = viewLoader.load("components/sidebar-search.component.fxml");

        GridPane.setColumnIndex(search, 1);
        GridPane.setRowIndex(search, 1);
        sidebar.getChildren().add(2, search);
    }

    /**
     * Activate the initial main category which is configured by the user.
     * This will select the category during startup of the application.
     */
    private void activateStartCategory() {
        var settings = applicationConfig.getSettings().getUiSettings();
        switchCategory(settings.getStartScreen(), true);
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
        if (icon == collectionText.getLabelFor()) {
            text = collectionText;
        }

        icon.getStyleClass().add(ACTIVE_STYLE);
        text.getStyleClass().add(ACTIVE_STYLE);
    }

    private void onCollectionActivated() {
        switchActiveItem(collectionIcon);
        eventPublisher.publish(new ShowTorrentCollectionEvent(this));
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

    private Transition createColorTransition() {
        return new Transition() {
            {
                setCycleCount(Animation.INDEFINITE);
                setCycleDuration(Duration.seconds(2));
                setAutoReverse(true);
            }

            @Override
            protected void interpolate(double frac) {
                var color = Color.rgb(36, 104, 204);
                infoIcon.setTextFill(color.interpolate(Color.rgb(45, 150, 217), frac));
            }
        };
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
            switchCategory((Icon) event.getTarget());
        }
    }

    @FXML
    void onCollectionClicked(MouseEvent event) {
        event.consume();
        onCollectionActivated();
    }

    @FXML
    void onCollectionPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            onCollectionActivated();
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
