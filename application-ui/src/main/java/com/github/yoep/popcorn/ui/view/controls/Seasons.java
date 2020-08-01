package com.github.yoep.popcorn.ui.view.controls;

import com.github.yoep.popcorn.ui.media.watched.controls.WatchedCell;
import com.github.yoep.popcorn.ui.media.watched.controls.WatchedCellFactory;
import com.github.yoep.popcorn.ui.view.models.Season;
import javafx.beans.property.ObjectProperty;
import javafx.beans.property.SimpleObjectProperty;
import javafx.scene.control.TableCell;
import javafx.scene.control.TableColumn;
import javafx.scene.control.TableView;
import org.springframework.util.Assert;

public class Seasons extends TableView<Season> {
    private static final String WATCHED_FACTORY_PROPERTY = "watchedFactory";

    private final ObjectProperty<WatchedCellFactory<Season>> watchedFactory =
            new SimpleObjectProperty<>(this, WATCHED_FACTORY_PROPERTY, WatchedCell::new);

    private final TableColumn<Season, String> seasonColumn = new TableColumn<>();
    private final TableColumn<Season, String> watchedColumn = new TableColumn<>();

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
     * Get the watched cell factory property.
     *
     * @return Returns the watched cell factory property.
     */
    public ObjectProperty<WatchedCellFactory<Season>> watchedFactoryProperty() {
        return watchedFactory;
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

    //region Functions

    private void init() {
        initializeColumns();
        initializeListeners();
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
        watchedColumn.setCellFactory(param -> getWatchedFactory().get());

        getColumns().add(seasonColumn);
        getColumns().add(watchedColumn);
    }

    private void initializeListeners() {
        watchedFactory.addListener((observable, oldValue, newValue) -> {
            watchedColumn.setCellFactory(param -> newValue.get());
            refresh();
        });
    }

    //endregion
}
