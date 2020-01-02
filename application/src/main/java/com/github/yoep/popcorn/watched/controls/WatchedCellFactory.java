package com.github.yoep.popcorn.watched.controls;

import com.github.yoep.popcorn.watched.models.Watchable;

import java.util.function.Supplier;

/**
 * Factory interface for creating a new watched cell.
 *
 * @param <T> The {@link Watchable} item type of the watched cell.
 */
public interface WatchedCellFactory<T extends Watchable> extends Supplier<WatchedCell<T>> {
}
