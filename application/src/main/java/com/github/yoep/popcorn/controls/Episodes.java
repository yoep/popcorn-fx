package com.github.yoep.popcorn.controls;

import com.github.yoep.popcorn.media.providers.models.Episode;
import com.github.yoep.popcorn.media.watched.controls.WatchedCell;
import com.github.yoep.popcorn.media.watched.controls.WatchedCellFactory;
import javafx.beans.property.ObjectProperty;
import javafx.beans.property.SimpleObjectProperty;
import javafx.scene.control.TableCell;
import javafx.scene.control.TableColumn;
import javafx.scene.control.TableView;
import org.springframework.util.Assert;

import java.util.Optional;

public class Episodes extends TableView<Episode> {
    private static final String WATCHED_FACTORY_PROPERTY = "watchedFactory";

    private final ObjectProperty<WatchedCellFactory<Episode>> watchedFactory =
            new SimpleObjectProperty<>(this, WATCHED_FACTORY_PROPERTY, WatchedCell::new);

    private final TableColumn<Episode, String> episodeColumn = new TableColumn<>();
    private final TableColumn<Episode, String> titleColumn = new TableColumn<>();
    private final TableColumn<Episode, String> watchedColumn = new TableColumn<>();

    //region Constructors

    public Episodes() {
        init();
    }

    //endregion

    //region Properties

    /**
     * Get the factory for creating new watched cells.
     *
     * @return Returns the watched cell factory.
     */
    public WatchedCellFactory<Episode> getWatchedFactory() {
        return watchedFactory.get();
    }

    /**
     * Get the watched cell factory property.
     *
     * @return Returns the watched cell factory property.
     */
    public ObjectProperty<WatchedCellFactory<Episode>> watchedFactoryProperty() {
        return watchedFactory;
    }

    /**
     * Set the new factory for creating watched cells.
     *
     * @param watchedFactory The new factory for creating watched cells (non-null).
     */
    public void setWatchedFactory(WatchedCellFactory<Episode> watchedFactory) {
        Assert.notNull(watchedFactory, "watchedFactory cannot be null");
        this.watchedFactory.set(watchedFactory);
    }

    //endregion

    //region Methods

    //endregion

    //region Functions

    private void init() {
        initializeColumns();
        initializeListeners();
    }

    private void initializeColumns() {
        episodeColumn.setMinWidth(50);
        episodeColumn.setMaxWidth(50);
        watchedColumn.setMinWidth(40);
        watchedColumn.setMaxWidth(40);

        episodeColumn.setCellFactory(param -> new TableCell<>() {
            @Override
            protected void updateItem(String item, boolean empty) {
                super.updateItem(item, empty);

                if (!empty) {
                    Optional.ofNullable(getTableRow().getItem())
                            .map(Episode::getEpisode)
                            .map(String::valueOf)
                            .ifPresent(this::setText);
                } else {
                    setText(null);
                    setGraphic(null);
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
                } else {
                    setText(null);
                    setGraphic(null);
                }
            }
        });
        watchedColumn.setCellFactory(param -> getWatchedFactory().get());

        getColumns().add(episodeColumn);
        getColumns().add(titleColumn);
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
