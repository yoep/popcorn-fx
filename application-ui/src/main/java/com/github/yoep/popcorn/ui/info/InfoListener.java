package com.github.yoep.popcorn.ui.info;

import com.github.yoep.popcorn.backend.info.ComponentInfo;

import java.util.List;

public interface InfoListener {
    /**
     * Invoked when the components details has been updated/changed.
     *
     * @param componentDetails The new list of component details.
     */
    void onComponentDetailsChanged(List<ComponentInfo> componentDetails);
}
