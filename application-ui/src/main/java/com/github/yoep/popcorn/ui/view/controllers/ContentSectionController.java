package com.github.yoep.popcorn.ui.view.controllers;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.backend.events.ErrorNotificationEvent;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.ShowDetailsEvent;
import com.github.yoep.popcorn.backend.settings.OptionsService;
import com.github.yoep.popcorn.ui.events.*;
import com.github.yoep.popcorn.ui.messages.ContentMessage;
import com.github.yoep.popcorn.ui.view.services.MaximizeService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.input.MouseButton;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.Pane;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.ApplicationEventPublisher;
import org.springframework.context.event.EventListener;

import java.net.URL;
import java.util.ResourceBundle;
import java.util.concurrent.atomic.AtomicReference;

@Slf4j
@RequiredArgsConstructor
public class ContentSectionController implements Initializable {
    private final ViewLoader viewLoader;
    private final LocaleText localeText;
    private final ApplicationEventPublisher applicationEventPublisher;
    private final EventPublisher eventPublisher;
    private final MaximizeService maximizeService;
    private final OptionsService optionsService;

    Pane detailsPane;
    Pane watchlistPane;
    Pane torrentCollectionPane;
    Pane settingsPane;
    Pane aboutPane;
    Pane updatePane;
    Pane rightTopSection;
    ContentType activeType;

    @FXML
    Pane contentPane;
    @FXML
    Pane listPane;

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

    @EventListener(ShowAboutEvent.class)
    public void onShowAbout() {
        switchContent(ContentType.ABOUT);
    }

    @EventListener(ShowUpdateEvent.class)
    public void onShowUpdate() {
        switchContent(ContentType.UPDATE);
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
        initializeContentPaneListener();
        initializePanes();
        initializeEventListeners();
        initializeMode();

        switchContent(ContentType.LIST);
    }

    private void initializeContentPaneListener() {
        contentPane.setOnMouseClicked(event -> {
            if (event.getSceneY() <= 40 && event.getButton() == MouseButton.PRIMARY && event.getClickCount() == 2) {
                event.consume();
                maximizeService.setMaximized(!maximizeService.isMaximized());
            }
        });
    }

    private void initializePanes() {
        // load the details pane on a different thread
        new Thread(() -> {
            detailsPane = viewLoader.load("common/sections/details.section.fxml");
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
        }, "content-loader").start();
    }

    private void initializeEventListeners() {
        eventPublisher.register(CategoryChangedEvent.class, event -> {
            switchContent(ContentType.LIST);
            return event;
        });
        eventPublisher.register(ShowSettingsEvent.class, event -> {
            switchContent(ContentType.SETTINGS);
            return event;
        });
    }

    private void initializeMode() {
        if (optionsService.isTvMode()) {
            rightTopSection = viewLoader.load("components/system-time.component.fxml");
        } else {
            rightTopSection = viewLoader.load("components/window.component.fxml");
        }

        AnchorPane.setTopAnchor(rightTopSection, 0.0);
        AnchorPane.setRightAnchor(rightTopSection, 0.0);
        contentPane.getChildren().add(2, rightTopSection);
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
            if (contentPane.getChildren().size() > 2)
                contentPane.getChildren().remove(0);

            try {
                contentPane.getChildren().add(0, pane.get());
            } catch (Exception ex) {
                log.error(ex.getMessage(), ex);
                applicationEventPublisher.publishEvent(new ErrorNotificationEvent(this, localeText.get(ContentMessage.CONTENT_PANE_FAILED)));
            }
        });
    }

    private void setAnchor(Pane pane) {
        AnchorPane.setTopAnchor(pane, 0.0);
        AnchorPane.setRightAnchor(pane, 0.0);
        AnchorPane.setBottomAnchor(pane, 0.0);
        AnchorPane.setLeftAnchor(pane, 0.0);
    }

    //endregion

    enum ContentType {
        LIST,
        DETAILS,
        WATCHLIST,
        TORRENT_COLLECTION,
        SETTINGS,
        ABOUT,
        UPDATE
    }
}
