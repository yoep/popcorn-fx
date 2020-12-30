package com.github.yoep.popcorn.ui.keys;

import javax.validation.constraints.NotNull;

public class DummyGlobalKeysService implements GlobalKeysService {
    @Override
    public void addListener(@NotNull GlobalKeysListener listener) {
        // no-op
    }

    @Override
    public void removeListener(GlobalKeysListener listener) {
        // no-op
    }
}
