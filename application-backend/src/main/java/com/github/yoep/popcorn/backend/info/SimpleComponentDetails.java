package com.github.yoep.popcorn.backend.info;

import lombok.Builder;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;

import java.beans.PropertyChangeListener;
import java.beans.PropertyChangeSupport;
import java.util.Objects;
import java.util.Optional;

@Getter
@EqualsAndHashCode(exclude = "changes")
@ToString(exclude = "changes")
public class SimpleComponentDetails implements ComponentInfo {
    private final String name;
    private String description;
    private ComponentState state;

    private final PropertyChangeSupport changes = new PropertyChangeSupport(this);

    @Builder
    public SimpleComponentDetails(String name, String description, ComponentState state) {
        Objects.requireNonNull(name, "name cannot be null");
        this.name = name;
        this.description = description;
        this.state = state;
    }

    @Override
    public Optional<String> getDescription() {
        return Optional.ofNullable(description);
    }

    public void setDescription(String description) {
        var oldValue = this.description;
        this.description = description;
        changes.firePropertyChange(DESCRIPTION_PROPERTY, oldValue, description);
    }

    public void setState(ComponentState state) {
        var oldValue = this.state;
        this.state = state;
        changes.firePropertyChange(STATE_PROPERTY, oldValue, state);
    }

    @Override
    public void addChangeListener(PropertyChangeListener listener) {
        Objects.requireNonNull(listener, "listener cannot be null");
        changes.addPropertyChangeListener(listener);
    }
}
