package com.github.yoep.popcorn.controllers.sections;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.LoadUrlActivity;
import com.github.yoep.popcorn.activities.ShowTorrentCollectionActivity;
import com.github.yoep.popcorn.torrent.TorrentCollectionService;
import com.github.yoep.popcorn.torrent.models.StoredTorrent;
import javafx.application.Platform;
import javafx.beans.property.SimpleObjectProperty;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.TableCell;
import javafx.scene.control.TableColumn;
import javafx.scene.control.TableView;
import javafx.scene.effect.BlurType;
import javafx.scene.effect.InnerShadow;
import javafx.scene.input.Clipboard;
import javafx.scene.input.ClipboardContent;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.Pane;
import javafx.scene.paint.Color;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Component;

import javax.annotation.PostConstruct;
import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
@Component
@RequiredArgsConstructor
public class TorrentCollectionSectionController implements Initializable {
    private final TorrentCollectionService torrentCollectionService;
    private final ActivityManager activityManager;

    @FXML
    private Pane fileShadow;
    @FXML
    private TableView<StoredTorrent> collection;

    //region Methods

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializeFileShadow();
        initializeCollection();
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    private void init() {
        activityManager.register(ShowTorrentCollectionActivity.class, activity -> onShowCollection());
    }

    //endregion

    //region Functions

    private void initializeFileShadow() {
        // inner shadows cannot be defined in CSS, so this needs to be done in code
        fileShadow.setEffect(new InnerShadow(BlurType.THREE_PASS_BOX, Color.color(0, 0, 0, 0.8), 10.0, 0.0, 0.0, 0.0));
    }

    private void initializeCollection() {
        TableColumn<StoredTorrent, Icon> magnetColumn = new TableColumn<>();
        TableColumn<StoredTorrent, String> nameColumn = new TableColumn<>();
        TableColumn<StoredTorrent, Icon> deleteColumn = new TableColumn<>();

        magnetColumn.setMaxWidth(40);
        magnetColumn.setMinWidth(40);
        magnetColumn.setCellValueFactory(item -> {
            Icon icon = new Icon(Icon.MAGNET_UNICODE);
            icon.setOnMouseClicked(event -> onMagnetClicked(event, item.getValue()));
            return new SimpleObjectProperty<>(icon);
        });
        nameColumn.setCellFactory(param -> {
            TableCell<StoredTorrent, String> cell = new TableCell<>() {
                @Override
                protected void updateItem(String item, boolean empty) {
                    super.updateItem(item, empty);
                    var rowItem = getTableRow().getItem();

                    if (!empty && rowItem != null) {
                        setText(rowItem.getName());
                    } else {
                        setText(null);
                    }
                }
            };
            cell.setOnMouseClicked(event -> onTorrentClicked(cell.getTableRow().getItem()));
            return cell;
        });
        deleteColumn.setMaxWidth(40);
        deleteColumn.setMinWidth(40);
        deleteColumn.setCellValueFactory(item -> {
            Icon icon = new Icon(Icon.TRASH_UNICODE);
            icon.setOnMouseClicked(event -> onDeleteClicked(event, item.getValue()));
            return new SimpleObjectProperty<>(icon);
        });

        collection.getColumns().add(magnetColumn);
        collection.getColumns().add(nameColumn);
        collection.getColumns().add(deleteColumn);
    }

    private void onShowCollection() {
        log.trace("Loading torrent collection list");
        Platform.runLater(() -> {
            collection.getItems().clear();
            collection.getItems().addAll(torrentCollectionService.getStoredTorrents());
        });
    }

    private void onTorrentClicked(StoredTorrent torrent) {
        activityManager.register((LoadUrlActivity) torrent::getMagnetUri);
    }

    private void onMagnetClicked(MouseEvent event, StoredTorrent item) {
        Clipboard clipboard = Clipboard.getSystemClipboard();
        ClipboardContent clipboardContent = new ClipboardContent();

        clipboardContent.putUrl(item.getMagnetUri());
        clipboardContent.putString(item.getMagnetUri());

        clipboard.setContent(clipboardContent);
        event.consume();
        log.debug("Magnet uri of {} has been copied to the clipboard", item);
    }

    private void onDeleteClicked(MouseEvent event, StoredTorrent item) {
        torrentCollectionService.removeTorrent(item.getMagnetUri());
        Platform.runLater(() -> collection.getItems().remove(item));
        event.consume();
    }

    //endregion
}
