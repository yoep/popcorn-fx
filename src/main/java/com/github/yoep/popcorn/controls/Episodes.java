package com.github.yoep.popcorn.controls;


import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.yoep.popcorn.media.providers.models.Episode;
import javafx.scene.control.TableCell;
import javafx.scene.control.TableColumn;
import javafx.scene.control.TableView;

import java.util.Optional;

public class Episodes extends TableView<Episode> {
    private static final String STYLE_CLASS_ICON = "icon";

    public Episodes() {
        initializeColumns();
    }

    private void initializeColumns() {
        TableColumn<Episode, String> episodeColumn = new TableColumn<>();
        TableColumn<Episode, String> titleColumn = new TableColumn<>();
        TableColumn<Episode, Icon> viewedColumn = new TableColumn<>();

        episodeColumn.setMinWidth(50);
        episodeColumn.setMaxWidth(50);
        viewedColumn.setMinWidth(40);
        viewedColumn.setMaxWidth(40);

        episodeColumn.setCellFactory(param -> new TableCell<>() {
            @Override
            protected void updateItem(String item, boolean empty) {
                super.updateItem(item, empty);

                if (!empty) {
                    Optional.ofNullable(getTableRow().getItem())
                            .map(Episode::getEpisode)
                            .map(String::valueOf)
                            .ifPresent(this::setText);
                }
            }
        });
        titleColumn.setCellFactory(param -> new TableCell<>() {
            @Override
            protected void updateItem(String item, boolean empty) {
                super.updateItem(item, empty);

                if (!empty) {
                    Optional.ofNullable(getTableRow().getItem())
                            .map(Episode::getTitle)
                            .ifPresent(this::setText);
                }
            }
        });
        viewedColumn.setCellFactory(param -> {
            Icon icon = new Icon();
            icon.getStyleClass().add(STYLE_CLASS_ICON);

            TableCell<Episode, Icon> cell = new TableCell<>() {
                @Override
                protected void updateItem(Icon item, boolean empty) {
                    super.updateItem(item, empty);

                    if (!empty) {
                        icon.setText(Icon.EYE_UNICODE);
                    }
                }
            };

            cell.setGraphic(icon);
            return cell;
        });

        this.getColumns().add(episodeColumn);
        this.getColumns().add(titleColumn);
        this.getColumns().add(viewedColumn);
    }
}
