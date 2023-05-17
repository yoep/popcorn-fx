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
            event_c.union.playerStopped_body = new EventC.PlayerStopped_Body();
            event_c.union.playerStopped_body.stoppedEvent = PlayerStoppedEventC.from(event);

            publishEvent(event_c);

            return event;
        }, EventPublisher.HIGHEST_ORDER);
//        eventPublisher.register(PlayVideoEvent.class, event -> {
//            var event_c = new EventC.ByValue();
//            event_c.tag = EventC.Tag.PlayVideo;
//            event_c.union = new EventC.EventCUnion.ByValue();
//            event_c.union.playVideo_body = new EventC.PlayVideo_Body();
//            event_c.union.playVideo_body.playVideoEvent = (event instanceof PlayMediaEvent mediaEvent)
//                    ? PlayVideoEventC.from(mediaEvent)
//                    : PlayVideoEventC.from(event);
//
//            publishEvent(event_c);
//
//            return event;
//        }, EventPublisher.HIGHEST_ORDER);
//        eventPublisher.register(PlayerStateEvent.class, event -> {
//            var event_c = new EventC.ByValue();
//            event_c.tag = EventC.Tag.PlaybackStateChanged;
//            event_c.union = new EventC.EventCUnion.ByValue();
//            event_c.union.playbackState_body = new EventC.PlaybackState_Body();
//            event_c.union.playbackState_body.newState = event.getNewState();
//
//            publishEvent(event_c);
//
//            return event;
//        }, EventPublisher.HIGHEST_ORDER);
    }

    private void publishEvent(EventC.ByValue event_c) {
        try (event_c) {
            log.debug("Sending FFI EventC for {}", event_c);
            fxLib.publish_event(instance, event_c);
        } catch (Exception ex) {
            log.error("An error occurred while publishing the FFI event, {}", ex.getMessage(), ex);
        }
    }
}
