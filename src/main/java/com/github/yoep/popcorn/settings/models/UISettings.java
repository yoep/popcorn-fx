package com.github.yoep.popcorn.settings.models;

import com.fasterxml.jackson.annotation.JsonIgnore;
import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Data;
import lombok.NoArgsConstructor;
import org.springframework.util.Assert;

import java.beans.PropertyChangeListener;
import java.beans.PropertyChangeSupport;
import java.util.Objects;

@Data
@Builder
@NoArgsConstructor
@AllArgsConstructor
public class UISettings {
    public static final String UI_SCALE_PROPERTY = "uiScale";

    @JsonIgnore
    protected final PropertyChangeSupport changes = new PropertyChangeSupport(this);

    /**
     * The ui scale of the application.
     */
    @Builder.Default
    private UIScale uiScale = new UIScale(1f);

    /**
     * Set the new UI scale value of the application.
     *
     * @param uiScale The new application ui scale.
     */
    public void setUiScale(UIScale uiScale) {
        if (Objects.equals(this.uiScale, uiScale))
            return;

        UIScale oldValue = this.uiScale;
        this.uiScale = uiScale;
        changes.firePropertyChange(UI_SCALE_PROPERTY, oldValue, uiScale);
    }

    /**
     * Register a new listener to this instance.
     *
     * @param listener The listener to add.
     */
    public void addListener(PropertyChangeListener listener) {
        Assert.notNull(listener, "listener cannot be null");
        changes.addPropertyChangeListener(listener);
    }

    /**
     * Remove an existing listener from this instance.
     *
     * @param listener The listener to remove.
     */
    public void removeListener(PropertyChangeListener listener) {
        Assert.notNull(listener, "listener cannot be null");
        changes.removePropertyChangeListener(listener);
    }
}
