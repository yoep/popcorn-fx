package com.github.yoep.popcorn.ui.view.controllers.desktop.sections;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.ui.events.LoadUrlEvent;
import com.github.yoep.popcorn.ui.events.ShowTorrentCollectionEvent;
import com.github.yoep.popcorn.ui.events.SuccessNotificationEvent;
import com.github.yoep.popcorn.ui.messages.TorrentMessage;
import com.github.yoep.popcorn.ui.torrent.TorrentCollectionService;
import com.github.yoep.popcorn.ui.torrent.controls.TorrentCollection;
import com.github.yoep.popcorn.ui.torrent.models.StoredTorrent;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.effect.BlurType;
import javafx.scene.effect.InnerShadow;
import javafx.scene.input.Clipboard;
import javafx.scene.input.ClipboardContent;
import javafx.scene.layout.Pane;
import javafx.scene.paint.Color;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.ApplicationEventPublisher;
import org.springframework.context.event.EventListener;

import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
@RequiredArgsConstructor
public class TorrentCollectionSectionController implements Initializable {
    private final ApplicationEventPublisher eventPublisher;
    private final TorrentCollectionService torrentCollectionService;
    private final LocaleText localeText;

    @FXML
    private Pane fileShadow;
    @FXML
    private TorrentCollection collection;

    //region Methods

    @EventListener(ShowTorrentCollectionEvent.class)
    public void onShowTorrentCollection() {
        log.trace("Loading torrent collection list");
        Platform.runLater(() -> {
            collection.getItems().clear();
            collection.getItems().addAll(torrentCollectionService.getStoredTorrents());
        });
    }

    //endregion

    //region Initializable

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializeFileShadow();
        initializeCollection();
    }

    //endregion

    //region Functions

    private void initializeFileShadow() {
        // inner shadows cannot be defined in CSS, so this needs to be done in code
        fileShadow.setEffect(new InnerShadow(BlurType.THREE_PASS_BOX, Color.color(0, 0, 0, 0.8), 10.0, 0.0, 0.0, 0.0));
    }

    private void initializeCollection() {
        collection.setOnMagnetClicked(this::onMagnetClicked);
        collection.setOnTorrentClicked(this::onTorrentClicked);
        collection.setOnDeleteClicked(this::onDeleteClicked);
    }

    private void onMagnetClicked(StoredTorrent item) {
        var clipboard = Clipboard.getSystemClipboard();
        var clipboardContent = new ClipboardContent();

        clipboardContent.putUrl(item.getMagnetUri());
        clipboardContent.putString(item.getMagnetUri());

        clipboard.setContent(clipboardContent);
        eventPublisher.publishEvent(new SuccessNotificationEvent(this, localeText.get(TorrentMessage.MAGNET_COPIED)));
        log.debug("Magnet uri of {} has been copied to the clipboard", item);
    }

    private void onTorrentClicked(StoredTorrent torrent) {
        eventPublisher.publishEvent(new LoadUrlEvent(this, torrent.getMagnetUri()));
    }

    private void onDeleteClicked(StoredTorrent item) {
        torrentCollectionService.removeTorrent(item.getMagnetUri());
        Platform.runLater(() -> collection.getItems().remove(item));
    }

    //endregion
}
