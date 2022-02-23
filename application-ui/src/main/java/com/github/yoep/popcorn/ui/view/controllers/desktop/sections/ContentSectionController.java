package com.github.yoep.popcorn.ui.view.controllers.desktop.sections;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.backend.events.ErrorNotificationEvent;
import com.github.yoep.popcorn.backend.events.ShowDetailsEvent;
import com.github.yoep.popcorn.ui.events.*;
import com.github.yoep.popcorn.ui.messages.ContentMessage;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.Pane;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.ApplicationEventPublisher;
import org.springframework.context.event.EventListener;
import org.springframework.core.task.TaskExecutor;

import java.net.URL;
import java.util.ResourceBundle;
import java.util.concurrent.atomic.AtomicReference;

@Slf4j
@RequiredArgsConstructor
public class ContentSectionController implements Initializable {
    private final ViewLoader viewLoader;
    private final TaskExecutor taskExecutor;
    private final LocaleText localeText;
    private final ApplicationEventPublisher eventPublisher;

    private Pane listPane;
    private Pane detailsPane;
    private Pane watchlistPane;
    private Pane torrentCollectionPane;
    private Pane settingsPane;
    private Pane aboutPane;
    private Pane updatePane;
    private ContentType activeType;

    @FXML
    private Pane contentPane;

    //region Methods

    @EventListener(ShowDetailsEvent.class)
    public void onShowDetails() {
        switchContent(ContentType.DETAILS);
    }

    @EventListener(ShowWatchlistEvent.class)
    public void onShowWatchlist() {
        switchContent(ContentType.WATCHLIST);
    }

    @EventListener(ShowTorrentCollectionEvent.class)
    public void onShowTorrentCollection() {
        switchContent(ContentType.TORRENT_COLLECTION);
    }

    @EventListener(ShowSettingsEvent.class)
    public void onShowSettings() {
        switchContent(ContentType.SETTINGS);
    }

    @EventListener(ShowAboutEvent.class)
    public void onShowAbout() {
        switchContent(ContentType.ABOUT);
    }

    @EventListener(ShowUpdateEvent.class)
    public void onShowUpdate() {
        switchContent(ContentType.UPDATE);
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
    public void initialize(URL location, ResourceBundle resources) {
        initializePanes();

        switchContent(ContentType.LIST);
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
            settingsPane = viewLoader.load("sections/settings.section.fxml");
            aboutPane = viewLoader.load("sections/about.section.fxml");
            updatePane = viewLoader.load("sections/update.section.fxml");

            setAnchor(detailsPane);
            setAnchor(torrentCollectionPane);
            setAnchor(watchlistPane);
            setAnchor(settingsPane);
            setAnchor(aboutPane);
            setAnchor(updatePane);
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
            case LIST -> pane.set(listPane);
            case DETAILS -> pane.set(detailsPane);
            case WATCHLIST -> pane.set(watchlistPane);
            case TORRENT_COLLECTION -> pane.set(torrentCollectionPane);
            case SETTINGS -> pane.set(settingsPane);
            case ABOUT -> pane.set(aboutPane);
            case UPDATE -> pane.set(updatePane);
        }

        Platform.runLater(() -> {
            if (contentPane.getChildren().size() > 1)
                contentPane.getChildren().remove(0);

            try {
                contentPane.getChildren().add(0, pane.get());
            } catch (Exception ex) {
                log.error(ex.getMessage(), ex);
                eventPublisher.publishEvent(new ErrorNotificationEvent(this, localeText.get(ContentMessage.CONTENT_PANE_FAILED)));
            }
        });
    }

    private void setAnchor(Pane pane) {
        AnchorPane.setTopAnchor(pane, 64d);
        AnchorPane.setRightAnchor(pane, 0d);
        AnchorPane.setBottomAnchor(pane, 0d);
        AnchorPane.setLeftAnchor(pane, 0d);
    }

    //endregion

    private enum ContentType {
        LIST,
        DETAILS,
        WATCHLIST,
        TORRENT_COLLECTION,
        SETTINGS,
        ABOUT,
        UPDATE
    }
}
