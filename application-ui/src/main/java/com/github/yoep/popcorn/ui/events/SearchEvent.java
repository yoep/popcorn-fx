package com.github.yoep.popcorn.ui.events;

import com.github.yoep.popcorn.backend.events.ApplicationEvent;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;

import java.util.Optional;

/**
 * Represents a search event that is triggered when a search activity occurs.
 */
@Getter
@ToString
@EqualsAndHashCode(callSuper = false)
public class SearchEvent extends ApplicationEvent {
    /**
     * The search value associated with this search event.
     */
    private final String value;

    /**
     * Constructs a new SearchEvent with the specified source and search value.
     *
     * @param source the object on which the event initially occurred
     * @param value  the search value associated with this search event; may be null
     */
    public SearchEvent(Object source, String value) {
        super(source);
        this.value = value;
    }

    /**
     * Retrieves the search value associated with this search event.
     *
     * @return an Optional containing the search value, or an empty Optional if the search value is null
     */
    public Optional<String> getValue() {
        return Optional.ofNullable(value);
    }
}
