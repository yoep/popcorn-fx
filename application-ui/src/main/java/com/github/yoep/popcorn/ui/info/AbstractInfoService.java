package com.github.yoep.popcorn.ui.info;

import com.github.yoep.popcorn.backend.info.ComponentInfo;

import javax.validation.constraints.NotNull;
import java.util.ArrayList;
import java.util.Collection;
import java.util.List;
import java.util.Objects;
import java.util.concurrent.ConcurrentLinkedQueue;

public abstract class AbstractInfoService {
    private final List<ComponentInfo> componentDetails = new ArrayList<>();
    private final Collection<InfoListener> listeners = new ConcurrentLinkedQueue<>();

    public List<ComponentInfo> getComponentDetails() {
        return componentDetails;
    }

    public void addListener(@NotNull InfoListener listener) {
        Objects.requireNonNull(listener, "listener cannot be null");
        listeners.add(listener);
    }

    protected void updateComponents(List<ComponentInfo> components) {
        componentDetails.clear();
        componentDetails.addAll(components);
        listeners.forEach(e -> e.onComponentDetailsChanged(componentDetails));
    }
}
