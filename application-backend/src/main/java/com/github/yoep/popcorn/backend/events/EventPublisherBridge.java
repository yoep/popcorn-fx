package com.github.yoep.popcorn.backend.events;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import lombok.extern.slf4j.Slf4j;

/**
 * The FFI bridge for all events that can be thrown across the application.
 */
@Slf4j
public class EventPublisherBridge {
    private final EventPublisher eventPublisher;
    private final FxLib fxLib;
    private final PopcornFx instance;

    public EventPublisherBridge(EventPublisher eventPublisher, FxLib fxLib, PopcornFx instance) {
        this.eventPublisher = eventPublisher;
        this.fxLib = fxLib;
        this.instance = instance;
        init();
    }

    private void init() {
        eventPublisher.register(PlayerStoppedEvent.class, event -> {
            try (var event_c = PlayerStoppedEventC.from(event)) {
                log.debug("Handling closed player event for auto-resume with {}", event_c);
                fxLib.handle_event(instance, event_c);
            }
            return event;
        }, EventPublisher.HIGHEST_ORDER);
        eventPublisher.register(PlayVideoEvent.class, event -> {
            
            return event;
        }, EventPublisher.HIGHEST_ORDER);
    }
}
