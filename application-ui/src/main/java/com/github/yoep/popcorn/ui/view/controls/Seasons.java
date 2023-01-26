package com.github.yoep.popcorn.ui.view.controls;

import com.github.yoep.popcorn.backend.media.filters.model.Season;
import com.github.yoep.popcorn.ui.controls.WatchedCell;
import com.github.yoep.popcorn.ui.controls.WatchedCellFactory;
import javafx.beans.property.ObjectProperty;
import javafx.beans.property.SimpleObjectProperty;
import javafx.collections.ListChangeListener;
import javafx.scene.control.TableCell;
import javafx.scene.control.TableColumn;
import javafx.scene.control.TableView;
import org.springframework.util.Assert;

import java.util.ArrayList;
import java.util.List;

public class Seasons extends TableView<Season> {
    private static final String WATCHED_FACTORY_PROPERTY = "watchedFactory";

    private final ObjectProperty<WatchedCellFactory<Season>> watchedFactory =
            new SimpleObjectProperty<>(this, WATCHED_FACTORY_PROPERTY, () -> new WatchedCell<>((media, newState, icon) -> {
            }));

    private final TableColumn<Season, String> seasonColumn = new TableColumn<>();
    private final TableColumn<Season, String> watchedColumn = new TableColumn<>();
    private final List<WatchedCell<Season>> watchedCells = new ArrayList<>();

    //region Constructors

    public Seasons() {
        init();
    }

    //endregion

    //region Properties

    /**
     * Get the factory for creating new watched cells.
     *
     * @return Returns the watched cell factory.
     */
    public WatchedCellFactory<Season> getWatchedFactory() {
        return watchedFactory.get();
    }

    /**
     * Set the new factory for creating watched cells.
     *
     * @param watchedFactory The new factory for creating watched cells (non-null).
     */
    public void setWatchedFactory(WatchedCellFactory<Season> watchedFactory) {
        Assert.notNull(watchedFactory, "watchedFactory cannot be null");
        this.watchedFactory.set(watchedFactory);
    }


    //endregion

    public void updateWatchedState(Season season, boolean newState) {
        watchedCells.stream()
                .filter(e -> e.getWatchableItem() == season)
                .findFirst()
                .ifPresent(e -> e.updateWatchedState(newState));
    }

    //region Functions

    private void init() {
        initializeColumns();
        initializeListeners();
        initializeCleanup();
    }

    private void initializeColumns() {
        watchedColumn.setMinWidth(40);
        watchedColumn.setMaxWidth(40);

        seasonColumn.setCellFactory(param -> new TableCell<>() {
            @Override
            protected void updateItem(String item, boolean empty) {
                super.updateItem(item, empty);
                Season rowItem = getTableRow().getItem();

                if (rowItem != null) {
                    setText(rowItem.getText());
                } else {
                    setText(null);
                }
            }
        });
        watchedColumn.setCellFactory(param -> {
            var cell = getWatchedFactory().get();
            watchedCells.add(cell);
            return cell;
        });

        getColumns().add(seasonColumn);
        getColumns().add(watchedColumn);
    }

    private void initializeListeners() {
        watchedFactory.addListener((observable, oldValue, newValue) -> {
            watchedColumn.setCellFactory(param -> {
                var cell = newValue.get();
                watchedCells.add(cell);
                return cell;
            });
            refresh();
        });
    }

    private void initializeCleanup() {
        getItems().addListener((ListChangeListener<? super Season>) change -> {
            while (change.next()) {
                for (var season : change.getRemoved()) {
                    watchedCells.removeIf(e -> e.getWatchableItem() == season);
                }
            }
        });
    }

    //endregion
}
