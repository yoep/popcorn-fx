package com.github.yoep.popcorn.ui.view.model;

import lombok.*;
import org.springframework.lang.Nullable;

import java.util.Optional;

@Getter
@Builder
@AllArgsConstructor
@EqualsAndHashCode
@ToString
public class AboutDetail {
    private final String name;
    @Nullable
    private final String description;
    private final State state;

    public Optional<String> getDescription() {
        return Optional.ofNullable(description);
    }

    public enum State {
        UNKNOWN,
        READY,
        ERROR
    }
}
