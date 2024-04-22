package com.github.yoep.popcorn.ui.view.controllers;

import com.github.yoep.popcorn.backend.events.ErrorNotificationEvent;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.ShowAboutEvent;
import com.github.yoep.popcorn.backend.events.ShowDetailsEvent;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.events.*;
import com.github.yoep.popcorn.ui.messages.ContentMessage;
import com.github.yoep.popcorn.ui.view.ViewLoader;
import com.github.yoep.popcorn.ui.view.services.MaximizeService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseButton;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.Pane;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.Optional;
import java.util.ResourceBundle;
import java.util.concurrent.atomic.AtomicReference;

@Slf4j
@RequiredArgsConstructor
public class ContentSectionController implements Initializable {
    static final String SYSTEM_TIME_COMPONENT = "components/system-time.component.fxml";
    static final String WINDOW_COMPONENT = "components/window.component.fxml";
    static final String SETTINGS_SECTION = "sections/settings.section.fxml";

    private final ViewLoader viewLoader;
    private final LocaleText localeText;
    private final EventPublisher eventPublisher;
    private final MaximizeService maximizeService;
    private final ApplicationConfig applicationConfig;

    Pane detailsPane;
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

    //region Initializable

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializePanes();
        initializeEventListeners();
        initializeMode();

        switchContent(ContentType.LIST);
    }

    private void initializePanes() {
        // load the details pane on a different thread
        new Thread(() -> {
            log.debug("Loading content panes");
            detailsPane = viewLoader.load("common/sections/details.section.fxml");
            torrentCollectionPane = viewLoader.load("common/sections/torrent-collection.section.fxml");
            settingsPane = viewLoader.load(SETTINGS_SECTION);
            aboutPane = viewLoader.load("common/sections/about.section.fxml");
            updatePane = viewLoader.load("common/sections/update.section.fxml");

            setAnchor(detailsPane);
            setAnchor(torrentCollectionPane);
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
        eventPublisher.register(CloseSettingsEvent.class, event -> {
            switchContent(ContentType.LIST);
            return event;
        });
        eventPublisher.register(ShowDetailsEvent.class, event -> {
            switchContent(ContentType.DETAILS);
            return event;
        });
        eventPublisher.register(ShowTorrentCollectionEvent.class, event -> {
            switchContent(ContentType.TORRENT_COLLECTION);
            return event;
        });
        eventPublisher.register(ShowAboutEvent.class, event -> {
            switchContent(ContentType.ABOUT);
            return event;
        });
        eventPublisher.register(ShowUpdateEvent.class, event -> {
            switchContent(ContentType.UPDATE);
            return event;
        });
        eventPublisher.register(CloseDetailsEvent.class, event -> {
            switchContent(ContentType.LIST);
            return event;
        });
        eventPublisher.register(CloseAboutEvent.class, event -> {
            switchContent(ContentType.LIST);
            return event;
        });
        eventPublisher.register(CloseUpdateEvent.class, event -> {
            switchContent(ContentType.LIST);
            return event;
        });
    }

    private void initializeMode() {
        if (applicationConfig.isTvMode()) {
            rightTopSection = viewLoader.load(SYSTEM_TIME_COMPONENT);
        } else {
            rightTopSection = viewLoader.load(WINDOW_COMPONENT);
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

        var pane = new AtomicReference<Pane>();
        this.activeType = contentType;

        switch (contentType) {
            case LIST -> pane.set(listPane);
            case DETAILS -> pane.set(detailsPane);
            case TORRENT_COLLECTION -> pane.set(torrentCollectionPane);
            case SETTINGS -> pane.set(settingsPane);
            case ABOUT -> pane.set(aboutPane);
            case UPDATE -> pane.set(updatePane);
        }

        Platform.runLater(() -> {
            if (contentPane.getChildren().size() > 2)
                contentPane.getChildren().remove(0);

            try {
                Optional.ofNullable(pane.get()).ifPresentOrElse(
                        e -> {
                            log.trace("Updating content pane to {}", e);
                            contentPane.getChildren().add(0, e);
                        },
                        () -> log.error("Failed to update content pane, pane is NULL")
                );
            } catch (Exception ex) {
                log.error(ex.getMessage(), ex);
                eventPublisher.publishEvent(new ErrorNotificationEvent(this, localeText.get(ContentMessage.CONTENT_PANE_FAILED)));
            }
        });
    }

    private void setAnchor(Pane pane) {
        AnchorPane.setTopAnchor(pane, 0.0);
        AnchorPane.setRightAnchor(pane, 0.0);
        AnchorPane.setBottomAnchor(pane, 0.0);
        AnchorPane.setLeftAnchor(pane, 0.0);
    }

    @FXML
    void onMouseClicked(MouseEvent event) {
        if (event.getSceneY() <= 40 && event.getButton() == MouseButton.PRIMARY && event.getClickCount() == 2) {
            event.consume();
            maximizeService.setMaximized(!maximizeService.isMaximized());
        }
    }

    @FXML
    void onKeyPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.HOME) {
            event.consume();
            eventPublisher.publish(new HomeEvent(this));
        } else if (event.getCode() == KeyCode.CONTEXT_MENU) {
            event.consume();
            eventPublisher.publish(new ContextMenuEvent(this));
        }
    }

    //endregion

    enum ContentType {
        LIST,
        DETAILS,
        TORRENT_COLLECTION,
        SETTINGS,
        ABOUT,
        UPDATE
    }
}
