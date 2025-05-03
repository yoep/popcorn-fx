package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.ShowTorrentDetailsEvent;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Playlist;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Subtitle;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Torrent;
import com.github.yoep.popcorn.backend.playlists.PlaylistManager;
import com.github.yoep.popcorn.backend.subtitles.ISubtitleInfo;
import com.github.yoep.popcorn.backend.subtitles.SubtitleInfoWrapper;
import com.github.yoep.popcorn.backend.subtitles.ISubtitleService;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.events.CloseTorrentDetailsEvent;
import com.github.yoep.popcorn.ui.messages.TorrentMessage;
import com.github.yoep.popcorn.backend.torrent.TorrentCollectionService;
import com.github.yoep.popcorn.ui.utils.WatchNowUtils;
import com.github.yoep.popcorn.ui.view.controls.PlayerDropDownButton;
import com.github.yoep.popcorn.ui.view.controls.SubtitleDropDownButton;
import com.github.yoep.popcorn.ui.view.services.SubtitlePickerService;
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
import java.util.Objects;
import java.util.ResourceBundle;

import static java.util.Arrays.asList;

@Slf4j
@RequiredArgsConstructor
public class DetailsTorrentComponent implements Initializable {
    private static final List<String> SUPPORTED_FILES = asList("mp4", "m4v", "avi", "mov", "mkv", "wmv");

    private final EventPublisher eventPublisher;
    private final TorrentCollectionService torrentCollectionService;
    private final LocaleText localeText;
    private final PlayerManagerService playerManagerService;
    private final SubtitlePickerService subtitlePickerService;
    private final ISubtitleService subtitleService;
    private final PlaylistManager playlistManager;

    Torrent.Info torrentInfo;
    ISubtitleInfo subtitleInfo;

    @FXML
    ListView<Torrent.Info.File> torrentList;
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
            var cell = new ListCell<Torrent.Info.File>() {
                @Override
                protected void updateItem(Torrent.Info.File item, boolean empty) {
                    super.updateItem(item, empty);
                    setText(empty ? null : item.getFilename());
                }
            };
            cell.setOnMouseClicked(this::onTorrentCellClicked);
            return cell;
        });
    }

    private void initializeSubtitleDropDown() {
        subtitleService.defaultSubtitles().thenAccept(subtitles -> Platform.runLater(() -> {
            subtitleButton.clear();
            subtitleButton.addDropDownItems(subtitles);
            subtitleButton.select(subtitles.getFirst());
            subtitleButton.selectedItemProperty().addListener((observable, oldValue, newValue) -> onSubtitleChanged(newValue));
        }));
    }

    //endregion

    //region Functions

    private void onTorrentCellClicked(MouseEvent event) {
        if (event.getSource() instanceof ListCell<?> cell) {
            if (cell.isEmpty())
                return;

            event.consume();
            onFileInfoClicked((Torrent.Info.File) cell.getItem());
        } else {
            log.warn("Expected a 'ListCell<>' but got '{}' instead", event.getSource().getClass());
        }
    }

    private void onShowTorrentDetails(ShowTorrentDetailsEvent event) {
        log.debug("Processing details of torrent info {}", event.getTorrentInfo().getName());
        this.torrentInfo = event.getTorrentInfo();
        this.subtitleInfo = null;
        var validFiles = torrentInfo.getFilesList().stream()
                .filter(e -> {
                    var extension = FilenameUtils.getExtension(e.getFilename());
                    return SUPPORTED_FILES.contains(extension.toLowerCase());
                })
                .sorted()
                .toList();

        subtitleService.defaultSubtitles()
                .thenApply(List::getFirst)
                .thenAccept(subtitle -> Platform.runLater(() -> {
                    torrentList.getItems().clear();
                    torrentList.getItems().addAll(validFiles);
                    subtitleButton.select(subtitle);
                    this.subtitleInfo = subtitle;
                }));

        torrentCollectionService.isStored(torrentInfo.getUri())
                .thenAccept(this::updateStoreTorrent);
    }

    private void onSubtitleChanged(ISubtitleInfo subtitleInfo) {
        Objects.requireNonNull(subtitleInfo, "subtitleInfo cannot be null");
        this.subtitleInfo = subtitleInfo;
        if (subtitleInfo.getLanguage() == Subtitle.Language.CUSTOM) {
            subtitlePickerService.pickCustomSubtitle()
                    .ifPresent(url -> {
                        this.subtitleInfo = new SubtitleInfoWrapper(Subtitle.Info.newBuilder()
                                .setLanguage(Subtitle.Language.CUSTOM)
                                .addFiles(Subtitle.Info.File.newBuilder()
                                        .setName(FilenameUtils.getBaseName(url))
                                        .setUrl(url)
                                        .build())
                                .build());
                        subtitleService.updatePreferredLanguage(Subtitle.Language.CUSTOM);
                    });
        } else {
            subtitleService.updatePreferredLanguage(subtitleInfo.getLanguage());
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
        this.torrentInfo = null;

        Platform.runLater(() -> torrentList.getItems().clear());
    }

    private void close() {
        reset();
        subtitleService.reset();
        eventPublisher.publishEvent(new CloseTorrentDetailsEvent(this));
    }

    void onFileInfoClicked(Torrent.Info.File fileInfo) {
        Objects.requireNonNull(fileInfo, "fileInfo cannot be null");
        playlistManager.play(Playlist.newBuilder()
                .addItems(Playlist.Item.newBuilder()
                        .setUrl(torrentInfo.getUri())
                        .setTitle(torrentInfo.getName())
                        .setSubtitlesEnabled(true)
                        .setTorrentFilename(fileInfo.getFilename())
                        .build())
                .build());
    }

    @FXML
    void onStoreOrRemoveTorrentClicked(MouseEvent event) {
        event.consume();
        torrentCollectionService.isStored(torrentInfo.getUri())
                .thenAccept(isStored -> {
                    if (isStored) {
                        torrentCollectionService.removeTorrent(torrentInfo.getUri());
                        updateStoreTorrent(false);
                    } else {
                        torrentCollectionService.addTorrent(torrentInfo);
                        updateStoreTorrent(true);
                    }
                });
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
