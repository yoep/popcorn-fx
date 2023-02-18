package com.github.yoep.popcorn.backend.info;

import javax.validation.constraints.NotNull;
import java.beans.PropertyChangeListener;
import java.util.Optional;

public interface ComponentInfo {
    String DESCRIPTION_PROPERTY = "description";
    String STATE_PROPERTY = "state";

    /**
     * Get the name of the component.
     *
     * @return Returns the name of the component.
     */
    @NotNull
    String getName();

    /**
     * Get the additional description information of the component.
     *
     * @return Returns the description of the component.
     */
    Optional<String> getDescription();

    /**
     * The state of the component it's currently in.
     *
     * @return Returns the state of the component.
     */
    ComponentState getState();

    /**
     * Add a listener which gets notified when the component info details have been changed.
     *
     * @param listener The listener to register.
     */
    void addChangeListener(@NotNull PropertyChangeListener listener);
}
