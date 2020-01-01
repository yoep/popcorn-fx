package com.github.yoep.popcorn.controls;


import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.yoep.popcorn.media.providers.models.Episode;
import javafx.beans.property.ObjectProperty;
import javafx.beans.property.SimpleObjectProperty;
import javafx.scene.control.TableCell;
import javafx.scene.control.TableColumn;
import javafx.scene.control.TableView;
import org.springframework.util.Assert;

import java.util.Optional;
import java.util.function.Supplier;

public class Episodes extends TableView<Episode> {
    private final ObjectProperty<Supplier<EpisodeWatchedCellFactory>> watchedFactory =
            new SimpleObjectProperty<>(this, "watchedFactory", EpisodeWatchedCellFactory::new);

    private final TableColumn<Episode, String> episodeColumn = new TableColumn<>();
    private final TableColumn<Episode, String> titleColumn = new TableColumn<>();
    private final TableColumn<Episode, Icon> watchedColumn = new TableColumn<>();

    //region Constructors

    public Episodes() {
        init();
    }

    //endregion

    //region Properties

    /**
     * Get the watched cell factory of this control.
     *
     * @return Returns the watched cell factory.
     */
    public Supplier<EpisodeWatchedCellFactory> getWatchedFactory() {
        return watchedFactory.get();
    }

    /**
     * Get the watched factory property of this control.
     *
     * @return Returns the watched factory property.
     */
    public ObjectProperty<Supplier<EpisodeWatchedCellFactory>> watchedFactoryProperty() {
        return watchedFactory;
    }

    /**
     * Set the watched factory for this control.
     *
     * @param watchedFactory The new watched factory.
     */
    public void setWatchedFactory(Supplier<EpisodeWatchedCellFactory> watchedFactory) {
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
