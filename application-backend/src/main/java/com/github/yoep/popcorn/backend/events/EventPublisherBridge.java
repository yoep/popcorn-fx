package com.github.yoep.popcorn.backend.events;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.ApplicationEvent;

/**
 * The FFI bridge for all events that can be thrown across the application.
 */
@Slf4j
public class EventPublisherBridge implements EventBridgeCallback {
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
            event_c.tag = EventC.Tag.PLAYER_STOPPED;
            event_c.union = new EventC.EventCUnion.ByValue();
            event_c.union.playerStopped_body = new EventC.PlayerStopped_Body();
            event_c.union.playerStopped_body.stoppedEvent = PlayerStoppedEventC.from(event);

            if (isBridgePublishAllowed(event))
                publishEvent(event_c);

            return event;
        }, EventPublisher.HIGHEST_ORDER);
        eventPublisher.register(PlayerStateEvent.class, event -> {
            var event_c = new EventC.ByValue();
            event_c.tag = EventC.Tag.PLAYBACK_STATE_CHANGED;
            event_c.union = new EventC.EventCUnion.ByValue();
            event_c.union.playbackState_body = new EventC.PlaybackState_Body();
            event_c.union.playbackState_body.newState = event.getNewState();

            if (isBridgePublishAllowed(event))
                publishEvent(event_c);

            return event;
        }, EventPublisher.HIGHEST_ORDER);

        log.debug("Registering event bridge callback");
        fxLib.register_event_callback(instance, this);
    }

    @Override
    public void callback(EventC.ByValue event) {
        try (event) {
            switch (event.getTag()) {
                case PLAYER_CHANGED,
                        PLAYER_STARTED,
                        PLAYER_STOPPED,
                        LOADING_STARTED,
                        LOADING_COMPLETED,
                        TORRENT_DETAILS_LOADED,
                        CLOSE_PLAYER -> eventPublisher.publish(event.toEvent());
                default -> log.warn("EventC callback of {} is currently not yet supported", event.getTag());
            }
        } catch (Exception ex) {
            log.error("An unexpected error occurred while processing the EventC, {}", ex.getMessage(), ex);
        }
    }

    private void publishEvent(EventC.ByValue event_c) {
        try (event_c) {
            log.debug("Sending FFI EventC for {}", event_c);
            fxLib.publish_event(instance, event_c);
        } catch (Exception ex) {
            log.error("An error occurred while publishing the FFI event, {}", ex.getMessage(), ex);
        }
    }

    private boolean isBridgePublishAllowed(ApplicationEvent event) {
        return !(event.getSource() instanceof EventC);
    }
}
