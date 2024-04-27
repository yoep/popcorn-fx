package com.github.yoep.popcorn.ui.view.controllers.common.sections;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.NotificationEvent;
import com.github.yoep.popcorn.ui.view.ViewLoader;
import com.github.yoep.popcorn.ui.view.controllers.common.components.NotificationComponent;
import javafx.animation.Animation;
import javafx.animation.TranslateTransition;
import javafx.application.Platform;
import javafx.event.ActionEvent;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.layout.Pane;
import javafx.util.Duration;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.Objects;
import java.util.Queue;
import java.util.ResourceBundle;
import java.util.concurrent.ConcurrentLinkedQueue;

@Slf4j
public class NotificationSectionController implements Initializable {
    static final String NOTIFICATION_VIEW = "common/components/notification.component.fxml";
    private static final int SAFETY_OFFSET = 20;

    private final ViewLoader viewLoader;
    private final EventPublisher eventPublisher;
    private final Queue<NotificationEvent> queue = new ConcurrentLinkedQueue<>();

    @FXML
    Pane rootPane;

    public NotificationSectionController(ViewLoader viewLoader, EventPublisher eventPublisher) {
        Objects.requireNonNull(viewLoader, "viewLoader cannot be null");
        Objects.requireNonNull(eventPublisher, "eventPublisher cannot be null");
        this.viewLoader = viewLoader;
        this.eventPublisher = eventPublisher;
        init();
    }

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        processQueue();
        rootPane.sceneProperty().addListener(observable -> processQueue());
    }

    //region Functions

    private void init() {
        eventPublisher.register(NotificationEvent.class, event -> {
            queue.add(event);
            processQueue();
            return event;
        });
    }

    private void processQueue() {
        if (rootPane == null || rootPane.getScene() == null)
            return;

        var event = queue.poll();
        if (event == null)
            return;

        displayNotification(event);
        processQueue();
    }

    private void displayNotification(NotificationEvent event) {
        var notificationPane = loadNotificationPane(event);
        var transition = new TranslateTransition(Duration.seconds(1), notificationPane);

        Platform.runLater(() -> {
            notificationPane.setVisible(false);
            rootPane.getChildren().add(notificationPane);

            notificationPane.widthProperty().addListener((observable, oldValue, newValue) -> {
                if (transition.getStatus() != Animation.Status.RUNNING) {
                    notificationPane.setTranslateX(newValue.doubleValue() + SAFETY_OFFSET);
                    transition.setToX(0);
                    transition.playFromStart();
                    notificationPane.setVisible(true);
                }
            });
        });
    }

    private Pane loadNotificationPane(NotificationEvent notificationActivity) {
        var controller = new NotificationComponent(notificationActivity);

        controller.setOnClose(this::closeNotification);

        return viewLoader.load(NOTIFICATION_VIEW, controller);
    }

    private void closeNotification(ActionEvent action) {
        var notificationPane = (Pane) action.getSource();
        var transition = new TranslateTransition(Duration.seconds(1), notificationPane);

        Platform.runLater(() -> {
            transition.setToX(notificationPane.getWidth() + SAFETY_OFFSET);
            transition.setOnFinished(event -> rootPane.getChildren().removeIf(e -> e == notificationPane));
            transition.playFromStart();
        });
    }

    //endregion
}
