package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentFileInfo;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentInfo;
import com.github.yoep.popcorn.ui.events.CloseTorrentDetailsEvent;
import com.github.yoep.popcorn.ui.events.LoadUrlTorrentEvent;
import com.github.yoep.popcorn.ui.events.ShowTorrentDetailsEvent;
import com.github.yoep.popcorn.ui.messages.TorrentMessage;
import com.github.yoep.popcorn.ui.torrent.TorrentCollectionService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Button;
import javafx.scene.control.ListView;
import javafx.scene.effect.BlurType;
import javafx.scene.effect.InnerShadow;
import javafx.scene.layout.Pane;
import javafx.scene.paint.Color;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.io.FilenameUtils;
import org.springframework.context.ApplicationEventPublisher;
import org.springframework.context.event.EventListener;

import java.net.URL;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.ResourceBundle;

import static java.util.Arrays.asList;

@Slf4j
@RequiredArgsConstructor
public class DetailsTorrentComponent implements Initializable {
    private static final List<String> SUPPORTED_FILES = asList("mp4", "m4v", "avi", "mov", "mkv", "wmv");

    private final ApplicationEventPublisher eventPublisher;
    private final TorrentCollectionService torrentCollectionService;
    private final LocaleText localeText;

    private String magnetUri;
    private TorrentInfo torrentInfo;

    @FXML
    private ListView<String> fileList;
    @FXML
    private Pane fileShadow;
    @FXML
    private Button storeTorrentButton;

    //region Methods

    @EventListener
    public void onShowTorrentDetails(ShowTorrentDetailsEvent event) {
        log.debug("Processing details of torrent info {}", event.getTorrentInfo().getName());
        this.magnetUri = event.getMagnetUri();
        this.torrentInfo = event.getTorrentInfo();
        var filenames = new ArrayList<String>();
        var files = torrentInfo.getFiles();

        for (TorrentFileInfo file : files) {
            var extension = FilenameUtils.getExtension(file.getFilename());

            if (SUPPORTED_FILES.contains(extension.toLowerCase()))
                filenames.add(file.getFilename());
        }

        // sort files to make it easier for the user
        filenames.sort(Comparator.comparing(String::toLowerCase));

        Platform.runLater(() -> {
            fileList.getItems().clear();
            fileList.getItems().addAll(filenames);
        });

        updateStoreTorrent(torrentCollectionService.isStored(magnetUri));
    }

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializeFileShadow();
        initializeFileList();
    }

    //endregion

    //region Functions

    private void initializeFileShadow() {
        // inner shadows cannot be defined in CSS, so this needs to be done in code
        fileShadow.setEffect(new InnerShadow(BlurType.THREE_PASS_BOX, Color.color(0, 0, 0, 0.8), 10.0, 0.0, 0.0, 0.0));
    }

    private void initializeFileList() {
        fileList.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> {
            if (newValue != null) {
                var files = torrentInfo.getFiles();

                files.stream()
                        .filter(e -> e.getFilename().equals(newValue))
                        .findFirst()
                        .ifPresentOrElse(
                                this::onFileClicked,
                                () -> log.error("Failed to find torrent file with name \"{}\"", newValue));
            }
        });
    }

    private void onFileClicked(TorrentFileInfo fileInfo) {
        eventPublisher.publishEvent(new LoadUrlTorrentEvent(this, torrentInfo, fileInfo));
    }

    private void updateStoreTorrent(boolean isStored) {
        String text;

        if (isStored) {
            text = localeText.get(TorrentMessage.REMOVE_COLLECTION);
        } else {
            text = localeText.get(TorrentMessage.STORE_COLLECTION);
        }

        Platform.runLater(() -> storeTorrentButton.setText(text));
    }

    private void reset() {
        this.magnetUri = null;
        this.torrentInfo = null;

        Platform.runLater(() -> fileList.getItems().clear());
    }

    private void close() {
        reset();

        eventPublisher.publishEvent(new CloseTorrentDetailsEvent(this));
    }

    @FXML
    private void onClose() {
        close();
    }

    @FXML
    private void onStoreClicked() {
        if (torrentCollectionService.isStored(magnetUri)) {
            torrentCollectionService.removeTorrent(magnetUri);
            updateStoreTorrent(false);
        } else {
            torrentCollectionService.addTorrent(magnetUri, torrentInfo);
            updateStoreTorrent(true);
        }
    }

    //endregion
}
