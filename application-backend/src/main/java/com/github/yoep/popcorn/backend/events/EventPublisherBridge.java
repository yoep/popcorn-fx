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
            var event_c = new EventC.ByValue();
            event_c.tag = EventC.Tag.PlayerStopped;
            event_c.union = new EventC.EventCUnion.ByValue();
            event_c.union.playerStoppedEventCBody = new EventC.PlayerStoppedEventCBody();
            event_c.union.playerStoppedEventCBody.stoppedEvent = PlayerStoppedEventC.from(event);

            try (event_c) {
                log.debug("Handling closed player event for auto-resume with {}", event_c);
                fxLib.publish_event(instance, event_c);
            }

            return event;
        }, EventPublisher.HIGHEST_ORDER);
        eventPublisher.register(PlayVideoEvent.class, event -> {
            return event;
        }, EventPublisher.HIGHEST_ORDER);
    }
}
