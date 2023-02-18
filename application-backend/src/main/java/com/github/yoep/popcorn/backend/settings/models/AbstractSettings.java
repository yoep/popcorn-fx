package com.github.yoep.popcorn.backend.settings.models;

import com.fasterxml.jackson.annotation.JsonIgnore;

import java.beans.PropertyChangeListener;
import java.beans.PropertyChangeSupport;
import java.util.Objects;

/**
 * Contains generic functionality that is shared between the different setting options.
 * This includes handling of defaults and changes to the settings.
 */
public abstract class AbstractSettings implements Settings {
    @JsonIgnore
    protected final PropertyChangeSupport changes = new PropertyChangeSupport(this);

    @Override
    public void addListener(PropertyChangeListener listener) {
        changes.addPropertyChangeListener(listener);
    }

    @Override
    public void removeListener(PropertyChangeListener listener) {
        changes.removePropertyChangeListener(listener);
    }

    //region Functions

    protected <T> T updateProperty(T oldValue, T newValue, String propertyName) {
        if (Objects.equals(oldValue, newValue)) {
            return newValue;
        }

        changes.firePropertyChange(propertyName, oldValue, newValue);
        return newValue;
    }

    //endregion
}
