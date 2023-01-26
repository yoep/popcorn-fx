package com.github.yoep.popcorn.ui.controls;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.watched.models.Watchable;
import javafx.scene.control.TableCell;
import lombok.extern.slf4j.Slf4j;

/**
 * An extension on the {@link TableCell} for controlling the "watched" property of a table row.
 *
 * @param <T> The {@link Watchable} item of the cell.
 */
@Slf4j
public class WatchedCell<T extends Media> extends TableCell<T, String> {
    private static final String STYLE_CLASS = "icon";
    private static final String WATCHED_STYLE_CLASS = "watched";

    private final Icon icon = new Icon(Icon.EYE_UNICODE);
    private final WatchedCellCallbacks cellCallbacks;
    private boolean isWatched;

    //region Constructors

    /**
     * Initialize a new instance of {@link WatchedCell}.
     */
    public WatchedCell(WatchedCellCallbacks cellCallbacks) {
        super();
        this.cellCallbacks = cellCallbacks;

        init();
    }

    //endregion

    //region Methods

    /**
     * Get the item of this watched cell.
     *
     * @return Returns the item of this cell.
     */
    public T getWatchableItem() {
        return getTableRow().getItem();
    }

    public void updateWatchedState(boolean newState) {
        updateIcon(newState);
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

        onItemChanged(newItem);
    }

    protected void onItemChanged(T newItem) {
        // no-op
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
                cellCallbacks.updateWatchedState(getTableRow().getItem(), !isWatched, icon);
            }
        });
    }

    protected void updateIcon(Boolean watched) {
        if (watched) {
            icon.getStyleClass().add(WATCHED_STYLE_CLASS);
            isWatched = true;
        } else {
            icon.getStyleClass().removeIf(e -> e.equals(WATCHED_STYLE_CLASS));
            isWatched = false;
        }
    }

    //endregion
}
