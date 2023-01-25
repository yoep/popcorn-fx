package com.github.yoep.popcorn.ui.controls;

import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.watched.models.Watchable;

import java.util.function.Supplier;

/**
 * Factory interface for creating a new watched cell.
 *
 * @param <T> The {@link Watchable} item type of the watched cell.
 */
public interface WatchedCellFactory<T extends Media> extends Supplier<WatchedCell<T>> {
}
