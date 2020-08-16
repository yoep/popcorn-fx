package com.github.yoep.popcorn.ui.view.controllers.tv.sections;

import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.ui.events.*;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.Pane;
import lombok.RequiredArgsConstructor;
import org.springframework.context.event.EventListener;
import org.springframework.core.task.TaskExecutor;

import java.net.URL;
import java.util.ResourceBundle;
import java.util.concurrent.atomic.AtomicReference;

@RequiredArgsConstructor
public class ContentSectionController implements Initializable {
    private final ViewLoader viewLoader;
    private final TaskExecutor taskExecutor;

    private Pane listPane;
    private Pane detailsPane;
    private Pane settingsPane;
    private ContentType activeType;

    @FXML
    private Pane contentPane;

    //region Methods

    @EventListener(ShowSettingsEvent.class)
    public void onShowSettings() {
        switchContent(ContentType.SETTINGS);
    }

    @EventListener(ShowDetailsEvent.class)
    public void onShowDetails() {
        switchContent(ContentType.DETAILS);
    }

    @EventListener(CategoryChangedEvent.class)
    public void onCategoryChanged() {
        switchContent(ContentType.LIST);
    }

    @EventListener(CloseDetailsEvent.class)
    public void onCloseDetails() {
        switchContent(ContentType.LIST);
    }

    @EventListener(CloseSettingsEvent.class)
    public void onCloseSettings() {
        switchContent(ContentType.LIST);
    }

    //endregion

    //region Initializable

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializePanes();
    }

    private void initializePanes() {
        listPane = viewLoader.load("sections/list.section.fxml");
        setAnchor(listPane);

        // load the details pane on a different thread
        taskExecutor.execute(() -> {
            detailsPane = viewLoader.load("sections/details.section.fxml");
            settingsPane = viewLoader.load("sections/settings.section.fxml");

            setAnchor(detailsPane);
            setAnchor(settingsPane);
        });
    }

    //endregion

    //region Functions

    private void switchContent(ContentType contentType) {
        if (activeType == contentType)
            return;

        AtomicReference<Pane> pane = new AtomicReference<>();
        this.activeType = contentType;

        switch (contentType) {
            case LIST:
                pane.set(listPane);
                break;
            case DETAILS:
                pane.set(detailsPane);
                break;
            case SETTINGS:
                pane.set(settingsPane);
                break;
        }

        Platform.runLater(() -> {
            if (contentPane.getChildren().size() > 1)
                contentPane.getChildren().remove(0);

            contentPane.getChildren().add(0, pane.get());
        });
    }

    private void setAnchor(Pane pane) {
        AnchorPane.setTopAnchor(pane, 0.0);
        AnchorPane.setRightAnchor(pane, 0.0);
        AnchorPane.setBottomAnchor(pane, 0.0);
        AnchorPane.setLeftAnchor(pane, 175.0);
    }

    //endregion

    private enum ContentType {
        LIST,
        DETAILS,
        SETTINGS
    }
}
