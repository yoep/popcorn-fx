package com.github.yoep.popcorn.torrent.controls;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.yoep.popcorn.torrent.models.StoredTorrent;
import javafx.beans.property.SimpleObjectProperty;
import javafx.scene.control.TableCell;
import javafx.scene.control.TableColumn;
import javafx.scene.control.TableView;
import javafx.scene.input.MouseEvent;

import java.util.function.Consumer;


public class TorrentCollection extends TableView<StoredTorrent> {
    private static final String ICON_CELL_STYLE_CLASS = "icon-cell";
    private static final String NAME_CELL_STYLE_CLASS = "name-cell";

    private final TableColumn<StoredTorrent, StoredTorrent> magnetColumn = new TableColumn<>();
    private final TableColumn<StoredTorrent, String> nameColumn = new TableColumn<>();
    private final TableColumn<StoredTorrent, StoredTorrent> deleteColumn = new TableColumn<>();

    private Consumer<StoredTorrent> magnetClickedConsumer;
    private Consumer<StoredTorrent> torrentClickedConsumer;
    private Consumer<StoredTorrent> deleteClickedConsumer;

    //region Constructors

    public TorrentCollection() {
        init();
    }

    //endregion

    //region Setters

    public void setOnMagnetClicked(Consumer<StoredTorrent> event) {
        magnetClickedConsumer = event;
    }

    public void setOnTorrentClicked(Consumer<StoredTorrent> event) {
        torrentClickedConsumer = event;
    }

    public void setOnDeleteClicked(Consumer<StoredTorrent> deleteClickedConsumer) {
        this.deleteClickedConsumer = deleteClickedConsumer;
    }

    //endregion

    //region Functions

    private void init() {
        initializeMagnetColumn();
        initializeNameColumn();
        initializeDeleteColumn();

        initializeColumns();
    }

    private void initializeColumns() {
        var columns = this.getColumns();

        columns.add(magnetColumn);
        columns.add(nameColumn);
        columns.add(deleteColumn);
    }

    private void initializeMagnetColumn() {
        magnetColumn.setMaxWidth(40);
        magnetColumn.setMinWidth(40);
        magnetColumn.setCellFactory(param -> {
            var cell = new TableCell<StoredTorrent, StoredTorrent>() {
                @Override
                protected void updateItem(StoredTorrent item, boolean empty) {
                    super.updateItem(item, empty);

                    if (!empty) {
                        setGraphic(new Icon(Icon.MAGNET_UNICODE));
                    } else {
                        setGraphic(null);
                    }
                }
            };

            cell.getStyleClass().add(ICON_CELL_STYLE_CLASS);
            cell.setOnMouseClicked(event -> onMagnetClicked(event, cell.getItem()));

            return cell;
        });
        magnetColumn.setCellValueFactory(param -> new SimpleObjectProperty<>(param.getValue()));
    }

    private void initializeNameColumn() {
        nameColumn.setCellFactory(param -> {
            var cell = new TableCell<StoredTorrent, String>() {
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

            cell.getStyleClass().add(NAME_CELL_STYLE_CLASS);
            cell.setOnMouseClicked(event -> onTorrentClicked(event, cell.getTableRow().getItem()));

            return cell;
        });
    }

    private void initializeDeleteColumn() {
        deleteColumn.setMaxWidth(40);
        deleteColumn.setMinWidth(40);
        deleteColumn.setCellFactory(item -> {
            var cell = new TableCell<StoredTorrent, StoredTorrent>() {
                @Override
                protected void updateItem(StoredTorrent item, boolean empty) {
                    super.updateItem(item, empty);

                    if (!empty) {
                        setGraphic(new Icon(Icon.TRASH_UNICODE));
                    } else {
                        setGraphic(null);
                    }
                }
            };

            cell.getStyleClass().add(ICON_CELL_STYLE_CLASS);
            cell.setOnMouseClicked(event -> onDeleteClicked(event, cell.getItem()));

            return cell;
        });
        deleteColumn.setCellValueFactory(param -> new SimpleObjectProperty<>(param.getValue()));
    }

    private void onMagnetClicked(MouseEvent event, StoredTorrent item) {
        if (magnetClickedConsumer != null) {
            event.consume();
            magnetClickedConsumer.accept(item);
        }
    }

    private void onTorrentClicked(MouseEvent event, StoredTorrent item) {
        if (torrentClickedConsumer != null) {
            event.consume();
            torrentClickedConsumer.accept(item);
        }
    }

    private void onDeleteClicked(MouseEvent event, StoredTorrent item) {
        if (deleteClickedConsumer != null) {
            event.consume();
            deleteClickedConsumer.accept(item);
        }
    }

    //endregion
}
