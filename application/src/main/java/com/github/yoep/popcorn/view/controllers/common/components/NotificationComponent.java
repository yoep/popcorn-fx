package com.github.yoep.popcorn.view.controllers.common.components;

import com.github.yoep.popcorn.activities.ErrorNotificationActivity;
import com.github.yoep.popcorn.activities.NotificationActivity;
import com.github.yoep.popcorn.activities.SuccessNotificationActivity;
import com.github.yoep.popcorn.activities.WarningNotificationActivity;
import javafx.animation.PauseTransition;
import javafx.event.ActionEvent;
import javafx.event.EventHandler;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import javafx.scene.layout.Pane;
import javafx.util.Duration;

import java.net.URL;
import java.util.ResourceBundle;

public class NotificationComponent implements Initializable {
    private static final String INFO_STYLE_CLASS = "info";
    private static final String SUCCESS_STYLE_CLASS = "success";
    private static final String WARNING_STYLE_CLASS = "warning";
    private static final String ERROR_STYLE_CLASS = "error";
    private static final Duration CLOSE_DELAY = Duration.seconds(5);

    private final PauseTransition pauseTransition = new PauseTransition(CLOSE_DELAY);
    private final NotificationActivity notificationActivity;

    @FXML
    private Pane rootPane;
    @FXML
    private Label text;

    private EventHandler<ActionEvent> onClose;

    public NotificationComponent(NotificationActivity notificationActivity) {
        this.notificationActivity = notificationActivity;
    }

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeBackground();
        initializeText();
        initializeCloseTransition();
    }

    /**
     * Set the action that needs to be exected when the notification is being closed.
     *
     * @param eventHandler The action handler.
     */
    public void setOnClose(EventHandler<ActionEvent> eventHandler) {
        onClose = eventHandler;
    }

    private void initializeBackground() {
        String styleClass;

        if (notificationActivity instanceof SuccessNotificationActivity) {
            styleClass = SUCCESS_STYLE_CLASS;
        } else if (notificationActivity instanceof WarningNotificationActivity) {
            styleClass = WARNING_STYLE_CLASS;
        } else if (notificationActivity instanceof ErrorNotificationActivity) {
            styleClass = ERROR_STYLE_CLASS;
        } else {
            styleClass = INFO_STYLE_CLASS;
        }

        rootPane.getStyleClass().add(styleClass);
    }

    private void initializeText() {
        text.setText(notificationActivity.getText());
    }

    private void initializeCloseTransition() {
        pauseTransition.setOnFinished(actionEvent -> close());

        rootPane.setOnMouseEntered(mouseEvent -> pauseTransition.stop());
        rootPane.setOnMouseExited(mouseEvent -> pauseTransition.playFromStart());

        pauseTransition.playFromStart();
    }

    @FXML
    private void close() {
        if (onClose != null)
            onClose.handle(new ActionEvent(rootPane, null));
    }
}
