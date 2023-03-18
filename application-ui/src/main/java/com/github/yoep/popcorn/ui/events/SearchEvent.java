package com.github.yoep.popcorn.ui.events;

import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;
import org.springframework.context.ApplicationEvent;
import org.springframework.lang.Nullable;

import java.util.Optional;

@Getter
@ToString
@EqualsAndHashCode(callSuper = false)
public class SearchEvent extends ApplicationEvent {
    /**
     * The search value of this search activity.
     */
    @Nullable
    private final String value;

    public SearchEvent(Object source, @Nullable String value) {
        super(source);
        this.value = value;
    }

    public Optional<String> getValue() {
        return Optional.ofNullable(value);
    }
}
