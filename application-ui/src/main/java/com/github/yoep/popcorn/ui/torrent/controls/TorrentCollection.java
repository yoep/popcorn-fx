package com.github.yoep.popcorn.ui.torrent.controls;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.MagnetInfo;
import com.github.yoep.popcorn.ui.font.controls.Icon;
import javafx.beans.property.SimpleObjectProperty;
import javafx.scene.control.TableCell;
import javafx.scene.control.TableColumn;
import javafx.scene.control.TableView;
import javafx.scene.input.MouseEvent;

import java.util.function.Consumer;

public class TorrentCollection extends TableView<MagnetInfo> {
    private static final String ICON_CELL_STYLE_CLASS = "icon-cell";
    private static final String NAME_CELL_STYLE_CLASS = "name-cell";

    private final TableColumn<MagnetInfo, MagnetInfo> magnetColumn = new TableColumn<>();
    private final TableColumn<MagnetInfo, String> nameColumn = new TableColumn<>();
    private final TableColumn<MagnetInfo, MagnetInfo> deleteColumn = new TableColumn<>();

    private Consumer<MagnetInfo> magnetClickedConsumer;
    private Consumer<MagnetInfo> torrentClickedConsumer;
    private Consumer<MagnetInfo> deleteClickedConsumer;

    //region Constructors

    public TorrentCollection() {
        init();
    }

    //endregion

    //region Properties

    public void setOnMagnetClicked(Consumer<MagnetInfo> event) {
        magnetClickedConsumer = event;
    }

    public Consumer<MagnetInfo> getTorrentClickedConsumer() {
        return torrentClickedConsumer;
    }

    public void setOnTorrentClicked(Consumer<MagnetInfo> event) {
        torrentClickedConsumer = event;
    }

    public void setOnDeleteClicked(Consumer<MagnetInfo> deleteClickedConsumer) {
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
        magnetColumn.setMaxWidth(50);
        magnetColumn.setMinWidth(50);
        magnetColumn.setCellFactory(param -> {
            var cell = new TableCell<MagnetInfo, MagnetInfo>() {
                @Override
                protected void updateItem(MagnetInfo item, boolean empty) {
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
            var cell = new TableCell<MagnetInfo, String>() {
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
        deleteColumn.setMaxWidth(50);
        deleteColumn.setMinWidth(50);
        deleteColumn.setCellFactory(item -> {
            var cell = new TableCell<MagnetInfo, MagnetInfo>() {
                @Override
                protected void updateItem(MagnetInfo item, boolean empty) {
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

    private void onMagnetClicked(MouseEvent event, MagnetInfo item) {
        if (magnetClickedConsumer != null) {
            event.consume();
            magnetClickedConsumer.accept(item);
        }
    }

    private void onTorrentClicked(MouseEvent event, MagnetInfo item) {
        if (item == null)
            return;

        if (torrentClickedConsumer != null) {
            event.consume();
            torrentClickedConsumer.accept(item);
        }
    }

    private void onDeleteClicked(MouseEvent event, MagnetInfo item) {
        if (deleteClickedConsumer != null) {
            event.consume();
            deleteClickedConsumer.accept(item);
        }
    }

    //endregion
}
