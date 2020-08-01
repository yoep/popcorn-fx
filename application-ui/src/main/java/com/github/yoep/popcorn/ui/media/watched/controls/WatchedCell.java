package com.github.yoep.popcorn.ui.media.watched.controls;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.yoep.popcorn.ui.media.watched.models.Watchable;
import javafx.beans.property.BooleanProperty;
import javafx.beans.value.ChangeListener;
import javafx.scene.control.TableCell;
import lombok.extern.slf4j.Slf4j;

import java.util.ArrayList;
import java.util.List;

/**
 * An extension on the {@link TableCell} for controlling the "watched" property of a table row.
 *
 * @param <T> The {@link Watchable} item of the cell.
 */
@Slf4j
public class WatchedCell<T extends Watchable> extends TableCell<T, String> {
    private static final String STYLE_CLASS = "icon";
    private static final String WATCHED_STYLE_CLASS = "watched";

    private final Icon icon = new Icon(Icon.EYE_UNICODE);
    private final List<ChangeListener<Boolean>> watchListeners = new ArrayList<>();
    private T oldItem;

    //region Constructors

    /**
     * Initialize a new instance of {@link WatchedCell}.
     */
    public WatchedCell() {
        super();

        init();
    }

    //endregion

    //region Properties

    /**
     * Check if this watchable item has been watched.
     *
     * @return Returns true if the item has been watched, else false.
     */
    public boolean isWatched() {
        return getWatchableItem().isWatched();
    }

    /**
     * Get the watched property of the watchable item.
     *
     * @return Returns the watched property of the item.
     */
    public BooleanProperty watchedProperty() {
        return getWatchableItem().watchedProperty();
    }

    /**
     * Set if the watchable item has been watched.
     *
     * @param watched The watched value for the item.
     */
    public void setWatched(boolean watched) {
        getWatchableItem().setWatched(watched);
    }

    //endregion

    //region Getters & Setters

    /**
     * Get the item of this watched cell.
     *
     * @return Returns the item of this cell.
     */
    public T getWatchableItem() {
        return getTableRow().getItem();
    }

    //endregion

    //region TableCell

    @Override
    protected void updateItem(String item, boolean empty) {
        super.updateItem(item, empty);
        T newItem = getTableRow().getItem();

        if (!empty) {
            setText("");
            setGraphic(icon);
        } else {
            setText(null);
            setGraphic(null);
        }

        onItemChanged(oldItem, newItem);
        oldItem = newItem;
    }

    //endregion

    //region Functions

    /**
     * Get the icon of this cell.
     *
     * @return Returns the icon of this cell.
     */
    protected Icon getIcon() {
        return icon;
    }

    /**
     * Invoked when the cell item is being changed.
     * Use this method when registering new listeners to the {@link Watchable#watchedProperty()} item.
     * Don't forget to remove the old listener from {@code oldItem}.
     *
     * @param oldItem The old {@link Watchable} item of the cell (can be null).
     * @param newItem The new {@link Watchable} item of the cell (can be null).
     */
    protected void onItemChanged(T oldItem, T newItem) {
        log.trace("Cell item is being changed from \"{}\" to \"{}\"", oldItem, newItem);
        if (oldItem != null && watchListeners.size() > 0) {
            log.trace("Removing old watched listeners from \"{}\"", oldItem);
            synchronized (watchListeners) {
                watchListeners.forEach(e -> oldItem.watchedProperty().removeListener(e));
                watchListeners.clear();
            }
        }

        if (newItem != null) {
            registerWatchedListener(createNewWatchedListener(), newItem);
        }
    }

    /**
     * Register a new watched listener to the {@link Watchable} item.
     * This is a convenient method to add new listeners as they're automatically being cleaned when the item is changed.
     *
     * @param listener The listener to register to the item.
     * @param item     The item to register the listener to.
     */
    protected void registerWatchedListener(ChangeListener<Boolean> listener, T item) {
        log.trace("Registering a new watched listener to \"{}\"", item);
        synchronized (watchListeners) {
            watchListeners.add(listener);
            item.watchedProperty().addListener(listener);
        }
    }

    private void init() {
        initializeIcon();
        initializeEvents();
    }

    private void initializeIcon() {
        icon.getStyleClass().add(STYLE_CLASS);
    }

    private void initializeEvents() {
        setOnMouseClicked(event -> {
            // check if graphic is being shown
            // if not, ignore the mouse click
            if (getGraphic() != null) {
                log.trace("Mouse has been clicked on \"{}\"", this);
                event.consume();
                setWatched(!isWatched());
            }
        });
    }

    private ChangeListener<Boolean> createNewWatchedListener() {
        log.trace("Creating new watched listener for \"{}\"", this);
        return (observable, oldValue, newValue) -> updateIcon(newValue);
    }

    protected void updateIcon(Boolean watched) {
        if (watched) {
            icon.getStyleClass().add(WATCHED_STYLE_CLASS);
        } else {
            icon.getStyleClass().removeIf(e -> e.equals(WATCHED_STYLE_CLASS));
        }
    }

    //endregion
}
