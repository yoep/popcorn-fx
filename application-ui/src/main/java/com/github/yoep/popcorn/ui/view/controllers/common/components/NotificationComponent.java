package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.yoep.popcorn.backend.events.ErrorNotificationEvent;
import com.github.yoep.popcorn.backend.events.NotificationEvent;
import com.github.yoep.popcorn.ui.events.SuccessNotificationEvent;
import com.github.yoep.popcorn.ui.events.WarningNotificationEvent;
import javafx.animation.PauseTransition;
import javafx.event.ActionEvent;
import javafx.event.EventHandler;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.Pane;
import javafx.util.Duration;

import java.net.URL;
import java.util.ResourceBundle;

public class NotificationComponent implements Initializable {
    static final String INFO_STYLE_CLASS = "info";
    static final String SUCCESS_STYLE_CLASS = "success";
    static final String WARNING_STYLE_CLASS = "warning";
    static final String ERROR_STYLE_CLASS = "error";
    static final Duration CLOSE_DELAY = Duration.seconds(5);

    private final PauseTransition pauseTransition = new PauseTransition(CLOSE_DELAY);
    private final NotificationEvent notificationActivity;

    @FXML
    Pane rootPane;
    @FXML
    Label text;

    private EventHandler<ActionEvent> onClose;

    public NotificationComponent(NotificationEvent notificationActivity) {
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

        if (notificationActivity instanceof SuccessNotificationEvent) {
            styleClass = SUCCESS_STYLE_CLASS;
        } else if (notificationActivity instanceof WarningNotificationEvent) {
            styleClass = WARNING_STYLE_CLASS;
        } else if (notificationActivity instanceof ErrorNotificationEvent) {
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
        pauseTransition.setOnFinished(actionEvent -> handleClose());

        rootPane.setOnMouseEntered(mouseEvent -> pauseTransition.stop());
        rootPane.setOnMouseExited(mouseEvent -> pauseTransition.playFromStart());

        pauseTransition.playFromStart();
    }

    private void handleClose() {
        if (onClose != null)
            onClose.handle(new ActionEvent(rootPane, null));
    }

    private void handleAction() {
        if (notificationActivity.getAction() == null)
            return;

        notificationActivity.getAction().run();
    }

    @FXML
    void onClicked(MouseEvent event) {
        event.consume();
        handleAction();
        handleClose();
    }
}
