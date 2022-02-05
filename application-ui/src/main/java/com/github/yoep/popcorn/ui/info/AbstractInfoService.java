package com.github.yoep.popcorn.ui.info;

import com.github.yoep.popcorn.backend.info.ComponentInfo;
import com.github.yoep.popcorn.backend.services.AbstractListenerService;

import java.util.ArrayList;
import java.util.List;

public abstract class AbstractInfoService extends AbstractListenerService<InfoListener> {
    private final List<ComponentInfo> componentDetails = new ArrayList<>();

    public List<ComponentInfo> getComponentDetails() {
        return componentDetails;
    }

    protected void updateComponents(List<ComponentInfo> components) {
        componentDetails.clear();
        componentDetails.addAll(components);
        listeners.forEach(e -> e.onComponentDetailsChanged(componentDetails));
    }
}
