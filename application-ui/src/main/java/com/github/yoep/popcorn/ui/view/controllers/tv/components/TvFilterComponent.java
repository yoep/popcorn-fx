package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.media.filters.model.Category;
import com.github.yoep.popcorn.backend.media.filters.model.Genre;
import com.github.yoep.popcorn.backend.media.filters.model.SortBy;
import com.github.yoep.popcorn.ui.events.CategoryChangedEvent;
import com.github.yoep.popcorn.ui.events.GenreChangeEvent;
import com.github.yoep.popcorn.ui.events.SortByChangeEvent;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import javax.annotation.PostConstruct;

@Slf4j
@RequiredArgsConstructor
public class TvFilterComponent {
    private final EventPublisher eventPublisher;
    private final FxLib fxLib;
    private final PopcornFx instance;

    @PostConstruct
    void init() {
        eventPublisher.register(CategoryChangedEvent.class, this::onCategoryChanged);
    }

    private CategoryChangedEvent onCategoryChanged(CategoryChangedEvent event) {
        updateGenres(event.getCategory());
        updateSortBy(event.getCategory());
        return event;
    }

    private void updateGenres(Category category) {
        try (var libGenres = fxLib.retrieve_provider_genres(instance, category.getProviderName())) {
            libGenres.values().stream()
                    .sorted()
                    .findFirst()
                    .ifPresent(e -> eventPublisher.publish(new GenreChangeEvent(this, new Genre(e, e))));
        }
    }

    private void updateSortBy(Category category) {
        try (var libSortBy = fxLib.retrieve_provider_sort_by(instance, category.getProviderName())) {
            libSortBy.values().stream()
                    .findFirst()
                    .ifPresent(e -> eventPublisher.publish(new SortByChangeEvent(this, new SortBy(e, e))));
        }
    }
}
