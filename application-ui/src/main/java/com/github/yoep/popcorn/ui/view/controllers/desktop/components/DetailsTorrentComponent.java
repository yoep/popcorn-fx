package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.spring.boot.javafx.stereotype.ViewController;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentFileInfo;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentInfo;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.subtitles.SubtitlePickerService;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import com.github.yoep.popcorn.ui.events.CloseTorrentDetailsEvent;
import com.github.yoep.popcorn.ui.events.LoadUrlTorrentEvent;
import com.github.yoep.popcorn.ui.events.ShowTorrentDetailsEvent;
import com.github.yoep.popcorn.ui.messages.TorrentMessage;
import com.github.yoep.popcorn.ui.torrent.TorrentCollectionService;
import com.github.yoep.popcorn.ui.utils.WatchNowUtils;
import com.github.yoep.popcorn.ui.view.controls.PlayerDropDownButton;
import com.github.yoep.popcorn.ui.view.controls.SubtitleDropDownButton;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Button;
import javafx.scene.control.ListCell;
import javafx.scene.control.ListView;
import javafx.scene.effect.BlurType;
import javafx.scene.effect.InnerShadow;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseButton;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.Pane;
import javafx.scene.paint.Color;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.io.FilenameUtils;

import java.net.URL;
import java.util.List;
import java.util.ResourceBundle;

import static java.util.Arrays.asList;

@Slf4j
@ViewController
@RequiredArgsConstructor
public class DetailsTorrentComponent implements Initializable {
    private static final List<String> SUPPORTED_FILES = asList("mp4", "m4v", "avi", "mov", "mkv", "wmv");

    private final EventPublisher eventPublisher;
    private final TorrentCollectionService torrentCollectionService;
    private final LocaleText localeText;
    private final PlayerManagerService playerManagerService;
    private final SubtitlePickerService subtitlePickerService;
    private final SubtitleService subtitleService;
    private final FxLib fxLib;

    private String magnetUri;
    private TorrentInfo torrentInfo;
    private SubtitleInfo activeSubtitleInfo;

    @FXML
    ListView<TorrentFileInfo> torrentList;
    @FXML
    Pane fileShadow;
    @FXML
    Button storeTorrentButton;
    @FXML
    PlayerDropDownButton playerButton;
    @FXML
    SubtitleDropDownButton subtitleButton;

    //region Initializable

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializeFileShadow();
        initializeFileList();
        initializeSubtitleDropDown();
        eventPublisher.register(ShowTorrentDetailsEvent.class, event -> {
            onShowTorrentDetails(event);
            return event;
        });

        WatchNowUtils.syncPlayerManagerAndWatchNowButton(playerManagerService, playerButton);
    }

    private void initializeFileShadow() {
        // inner shadows cannot be defined in CSS, so this needs to be done in code
        fileShadow.setEffect(new InnerShadow(BlurType.THREE_PASS_BOX, Color.color(0, 0, 0, 0.8), 10.0, 0.0, 0.0, 0.0));
    }

    private void initializeFileList() {
        torrentList.setCellFactory(view -> {
            var cell = new ListCell<TorrentFileInfo>() {
                @Override
                protected void updateItem(TorrentFileInfo item, boolean empty) {
                    super.updateItem(item, empty);
                    setText(empty ? null : item.getFilename());
                }
            };
            cell.setOnMouseClicked(this::onTorrentCellClicked);
            return cell;
        });
    }

    private void initializeSubtitleDropDown() {
        subtitleButton.addDropDownItems(fxLib.subtitle_none(), fxLib.subtitle_custom());
        subtitleButton.select(fxLib.subtitle_none());
        subtitleButton.selectedItemProperty().addListener((observable, oldValue, newValue) -> onSubtitleChanged(newValue));
    }

    //endregion

    //region Functions

    private void onTorrentCellClicked(MouseEvent event) {
        var cell = (ListCell<String>) event.getSource();
        if (cell.isEmpty())
            return;

        event.consume();
        handleFileChanged(cell.getItem());
    }

    private void onShowTorrentDetails(ShowTorrentDetailsEvent event) {
        log.debug("Processing details of torrent info {}", event.getTorrentInfo().getName());
        this.magnetUri = event.getMagnetUri();
        this.torrentInfo = event.getTorrentInfo();
        var validFiles = torrentInfo.getFiles().stream()
                .filter(e -> {
                    var extension = FilenameUtils.getExtension(e.getFilename());
                    return SUPPORTED_FILES.contains(extension.toLowerCase());
                })
                .sorted()
                .toList();

        Platform.runLater(() -> {
            torrentList.getItems().clear();
            torrentList.getItems().addAll(validFiles);
        });

        updateStoreTorrent(torrentCollectionService.isStored(magnetUri));
    }

    private void handleFileChanged(String filename) {
        var files = torrentInfo.getFiles();

        files.stream()
                .filter(e -> e.getFilename().equals(filename))
                .findFirst()
                .ifPresentOrElse(
                        this::onFileClicked,
                        () -> log.error("Failed to find torrent file with name \"{}\"", filename));

        // reset the file selection for later use
        torrentList.getSelectionModel().clearSelection();
    }

    private void onFileClicked(TorrentFileInfo fileInfo) {
        eventPublisher.publishEvent(new LoadUrlTorrentEvent(this, torrentInfo, fileInfo, activeSubtitleInfo));
    }

    private void onSubtitleChanged(SubtitleInfo subtitleInfo) {
        if (subtitleInfo.isCustom()) {
            subtitlePickerService.pickCustomSubtitle().ifPresent(subtitleService::updateCustomSubtitle);
        } else {
            activeSubtitleInfo = subtitleInfo;
        }
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

        Platform.runLater(() -> torrentList.getItems().clear());
    }

    private void close() {
        reset();
        eventPublisher.publishEvent(new CloseTorrentDetailsEvent(this));
    }

    @FXML
    void onStoreOrRemoveTorrentClicked(MouseEvent event) {
        event.consume();
        if (torrentCollectionService.isStored(magnetUri)) {
            torrentCollectionService.removeTorrent(magnetUri);
            updateStoreTorrent(false);
        } else {
            torrentCollectionService.addTorrent(magnetUri, torrentInfo);
            updateStoreTorrent(true);
        }
    }

    @FXML
    void onKeyPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.BACK_SPACE) {
            event.consume();
            close();
        }
    }

    @FXML
    void onMouseClicked(MouseEvent event) {
        if (event.getButton() == MouseButton.BACK) {
            event.consume();
            close();
        }
    }

    //endregion
}
