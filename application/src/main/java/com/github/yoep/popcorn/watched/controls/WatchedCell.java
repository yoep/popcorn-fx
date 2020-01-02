package com.github.yoep.popcorn.watched.controls;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.yoep.popcorn.watched.models.Watchable;
import javafx.beans.property.BooleanProperty;
import javafx.beans.property.SimpleBooleanProperty;
import javafx.scene.control.TableCell;

/**
 * An extension on the {@link TableCell} for controlling the "watched" property of a table row.
 *
 * @param <T> The {@link Watchable} item of the cell.
 */
public class WatchedCell<T extends Watchable> extends TableCell<T, Icon> {
    private static final String STYLE_CLASS = "icon";
    private static final String WATCHED_STYLE_CLASS = "watched";

    private final Icon icon = new Icon(Icon.EYE_UNICODE);
    private final BooleanProperty watched = new SimpleBooleanProperty(this, "watched", false);

    //region Constructors

    /**
     * Initialize a new instance of {@link WatchedCell}.
     */
    public WatchedCell() {
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
        return watched.get();
    }

    /**
     * Get the watched property of the watchable item.
     *
     * @return Returns the watched property of the item.
     */
    public BooleanProperty watchedProperty() {
        return watched;
    }

    /**
     * Set if the watchable item has been watched.
     *
     * @param watched The watched value for the item.
     */
    public void setWatched(boolean watched) {
        this.watched.set(watched);
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
    protected void updateItem(Icon item, boolean empty) {
        super.updateItem(item, empty);

        if (!empty) {
            setGraphic(icon);
        }
    }

    //endregion

    //region Functions

    protected Icon getIcon() {
        return icon;
    }

    protected void init() {
        initializeIcon();
        initializeListeners();
        initializeEvents();
    }

    private void initializeIcon() {
        icon.getStyleClass().add(STYLE_CLASS);
    }

    private void initializeListeners() {
        watched.addListener((observable, oldValue, newValue) -> {
            if (newValue) {
                icon.getStyleClass().add(WATCHED_STYLE_CLASS);
            } else {
                icon.getStyleClass().remove(WATCHED_STYLE_CLASS);
            }
        });
    }

    private void initializeEvents() {
        setOnMouseClicked(event -> {
            // check if graphic is being shown
            // if not, ignore the mouse click
            if (getGraphic() != null) {
                event.consume();
                setWatched(!isWatched());
            }
        });
    }

    //endregion
}
