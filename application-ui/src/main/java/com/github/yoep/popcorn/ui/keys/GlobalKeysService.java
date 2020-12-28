package com.github.yoep.popcorn.ui.keys;

import javax.validation.constraints.NotNull;

public interface GlobalKeysService {
    /**
     * Add the given listener to the global keys service.
     *
     * @param listener The listener to add.
     */
    void addListener(@NotNull GlobalKeysListener listener);

    /**
     * Remove the given listener from the global keys service.
     *
     * @param listener The listener to remove.
     */
    void removeListener(GlobalKeysListener listener);
}
