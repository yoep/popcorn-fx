package com.github.yoep.popcorn.controls;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.yoep.popcorn.media.providers.models.Episode;
import javafx.beans.property.BooleanProperty;
import javafx.beans.property.SimpleBooleanProperty;
import javafx.scene.control.TableCell;

public class EpisodeWatchedCellFactory extends TableCell<Episode, Icon> {
    private static final String STYLE_CLASS = "icon";
    private static final String WATCHED_STYLE_CLASS = "watched";

    private final Icon icon = new Icon(Icon.EYE_UNICODE);
    private final BooleanProperty watched = new SimpleBooleanProperty(this, "watched", false);

    //region Constructors

    public EpisodeWatchedCellFactory() {
        init();
    }

    //endregion

    //region Properties

    /**
     * Get if this episode cell has been watched.
     *
     * @return Returns true if this episode has been watched, else false.
     */
    public boolean isWatched() {
        return watched.get();
    }

    /**
     * Get the watched property of this cell.
     *
     * @return Returns the watched property.
     */
    public BooleanProperty watchedProperty() {
        return watched;
    }

    /**
     * Set if the episode has been watched.
     *
     * @param watched Set the watched value of the episode.
     */
    public void setWatched(boolean watched) {
        this.watched.set(watched);
    }

    /**
     * Get the episode of this cell.
     *
     * @return Returns the episode of this cell.
     */
    public Episode getEpisode() {
        return getTableRow().getItem();
    }

    //endregion

    //region TableCell

    @Override
    protected void updateItem(Icon item, boolean empty) {
        if (!empty) {
            setGraphic(icon);
        }
    }

    //endregion

    //region Functions

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
            event.consume();
            setWatched(!isWatched());
        });
    }

    //endregion
}
