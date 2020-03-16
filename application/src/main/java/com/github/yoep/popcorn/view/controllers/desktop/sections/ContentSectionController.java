package com.github.yoep.popcorn.view.controllers.desktop.sections;

import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.activities.*;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.Pane;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.task.TaskExecutor;

import java.net.URL;
import java.util.ResourceBundle;
import java.util.concurrent.atomic.AtomicReference;

@Slf4j
@RequiredArgsConstructor
public class ContentSectionController implements Initializable {
    private final ActivityManager activityManager;
    private final ViewLoader viewLoader;
    private final TaskExecutor taskExecutor;

    private Pane listPane;
    private Pane detailsPane;
    private Pane watchlistPane;
    private Pane torrentCollectionPane;
    private ContentType activeType;

    @FXML
    private Pane rootPane;

    //region Initializable

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializeListeners();
        initializePanes();

        switchContent(ContentType.LIST);
    }

    private void initializeListeners() {
        activityManager.register(ShowDetailsActivity.class, activity -> switchContent(ContentType.DETAILS));
        activityManager.register(CloseDetailsActivity.class, activity -> switchContent(ContentType.LIST));
        activityManager.register(ShowWatchlistActivity.class, activity -> switchContent(ContentType.WATCHLIST));
        activityManager.register(ShowTorrentCollectionActivity.class, activity -> switchContent(ContentType.TORRENT_COLLECTION));
        activityManager.register(CategoryChangedActivity.class, activity -> switchContent(ContentType.LIST));
    }

    private void initializePanes() {
        // load the list pane on the main thread
        // this blocks Spring from completing the startup stage while this pane is being loaded
        listPane = viewLoader.load("sections/list.section.fxml");
        setAnchor(listPane);

        // load the details pane on a different thread
        taskExecutor.execute(() -> {
            detailsPane = viewLoader.load("sections/details.section.fxml");
            torrentCollectionPane = viewLoader.load("sections/torrent-collection.section.fxml");
            watchlistPane = viewLoader.load("sections/watchlist.section.fxml");

            setAnchor(detailsPane);
            setAnchor(torrentCollectionPane);
            setAnchor(watchlistPane);
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
            case WATCHLIST:
                pane.set(watchlistPane);
                break;
            case TORRENT_COLLECTION:
                pane.set(torrentCollectionPane);
                break;
        }

        Platform.runLater(() -> {
            if (rootPane.getChildren().size() > 1)
                rootPane.getChildren().remove(0);

            rootPane.getChildren().add(0, pane.get());
        });
    }

    private void setAnchor(Pane pane) {
        AnchorPane.setTopAnchor(pane, 50d);
        AnchorPane.setRightAnchor(pane, 0d);
        AnchorPane.setBottomAnchor(pane, 0d);
        AnchorPane.setLeftAnchor(pane, 0d);
    }

    //endregion

    private enum ContentType {
        LIST,
        DETAILS,
        WATCHLIST,
        TORRENT_COLLECTION
    }
}
