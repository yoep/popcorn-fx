package com.github.yoep.popcorn.ui.player;

import com.github.yoep.player.adapter.PlayRequest;
import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Getter;

import java.util.Optional;

@Getter
@Builder
@AllArgsConstructor
public class SimplePlayRequest implements PlayRequest {
    private final String url;
    private String title;

    @Override
    public Optional<String> getTitle() {
        return Optional.ofNullable(title);
    }
}
