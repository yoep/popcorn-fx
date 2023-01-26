package com.github.yoep.popcorn.ui.view.controls;

import com.github.yoep.popcorn.backend.media.providers.models.Episode;
import com.github.yoep.popcorn.ui.controls.WatchedCell;
import com.github.yoep.popcorn.ui.controls.WatchedCellFactory;
import javafx.beans.property.ObjectProperty;
import javafx.beans.property.SimpleObjectProperty;
import javafx.collections.ListChangeListener;
import javafx.scene.control.SelectionMode;
import javafx.scene.control.TableCell;
import javafx.scene.control.TableColumn;
import javafx.scene.control.TableView;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import lombok.extern.slf4j.Slf4j;
import org.springframework.util.Assert;

import java.awt.*;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

@Slf4j
public class Episodes extends TableView<Episode> {
    private static final String WATCHED_FACTORY_PROPERTY = "watchedFactory";

    private final ObjectProperty<WatchedCellFactory<Episode>> watchedFactory =
            new SimpleObjectProperty<>(this, WATCHED_FACTORY_PROPERTY, () -> new WatchedCell<>((media, newState, icon) -> {
                // no-op
            }));

    private final TableColumn<Episode, String> episodeColumn = new TableColumn<>();
    private final TableColumn<Episode, String> titleColumn = new TableColumn<>();
    private final TableColumn<Episode, String> watchedColumn = new TableColumn<>();
    private final List<WatchedCell<Episode>> watchedCells = new ArrayList<>();

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

    @Override
    public void requestFocus() {
        super.requestFocus();
    }

    public void updateWatchedState(Episode episode, boolean newState) {
        watchedCells.stream()
                .filter(e -> e.getWatchableItem() == episode)
                .findFirst()
                .ifPresent(e -> e.updateWatchedState(newState));
    }

    //endregion

    //region Functions

    private void init() {
        initializeColumns();
        initializeListeners();
        initializeKeyEvents();
        initializeFocus();
        initializeCleanup();
    }

    private void initializeCleanup() {
        getItems().addListener((ListChangeListener<? super Episode>) change -> {
            while (change.next()) {
                for (var episode : change.getRemoved()) {
                    watchedCells.removeIf(e -> e.getWatchableItem() == episode);
                }
            }
        });
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
        watchedColumn.setCellFactory(param -> {
            var cell = getWatchedFactory().get();
            watchedCells.add(cell);
            return cell;
        });

        getColumns().add(episodeColumn);
        getColumns().add(titleColumn);
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

    private void initializeKeyEvents() {
        try {
            var robot = new Robot();
            var focusMoveCode = KeyCode.TAB.getCode();
            var previousCode = KeyCode.SHIFT.getCode();

            this.addEventHandler(KeyEvent.KEY_PRESSED, event -> {
                if (event.getCode() == KeyCode.RIGHT) {
                    event.consume();

                    robot.keyPress(focusMoveCode);
                    robot.keyRelease(focusMoveCode);
                } else if (event.getCode() == KeyCode.LEFT) {
                    event.consume();

                    robot.keyPress(previousCode);
                    robot.keyPress(focusMoveCode);
                    robot.keyRelease(previousCode);
                    robot.keyRelease(focusMoveCode);
                }
            });
        } catch (AWTException ex) {
            log.error("Failed to create episodes robot, " + ex.getMessage(), ex);
        }
    }

    private void initializeFocus() {
        getSelectionModel().setCellSelectionEnabled(false);
        getSelectionModel().setSelectionMode(SelectionMode.SINGLE);
    }

    //endregion
}
