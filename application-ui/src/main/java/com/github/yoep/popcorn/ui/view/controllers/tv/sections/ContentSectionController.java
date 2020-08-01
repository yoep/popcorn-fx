package com.github.yoep.popcorn.ui.view.controllers.tv.sections;

import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.ui.activities.*;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.Pane;
import lombok.RequiredArgsConstructor;
import org.springframework.core.task.TaskExecutor;

import javax.annotation.PostConstruct;
import java.net.URL;
import java.util.ResourceBundle;
import java.util.concurrent.atomic.AtomicReference;

@RequiredArgsConstructor
public class ContentSectionController implements Initializable {
    private final ActivityManager activityManager;
    private final ViewLoader viewLoader;
    private final TaskExecutor taskExecutor;

    private Pane listPane;
    private Pane detailsPane;
    private Pane settingsPane;
    private ContentType activeType;

    @FXML
    private Pane contentPane;

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

    //region PostConstruct

    @PostConstruct
    private void init() {
        initializeListeners();
    }

    private void initializeListeners() {
        activityManager.register(ShowSettingsActivity.class, activity -> switchContent(ContentType.SETTINGS));
        activityManager.register(ShowDetailsActivity.class, activity -> switchContent(ContentType.DETAILS));
        activityManager.register(CategoryChangedActivity.class, activity -> switchContent(ContentType.LIST));

        activityManager.register(CloseDetailsActivity.class, activity -> switchContent(ContentType.LIST));
        activityManager.register(CloseSettingsActivity.class, activity -> switchContent(ContentType.LIST));
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
