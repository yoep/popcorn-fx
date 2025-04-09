package com.github.yoep.popcorn.backend.events;

import com.github.yoep.popcorn.backend.lib.FxCallback;
import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Event;
import lombok.extern.slf4j.Slf4j;

import java.util.Objects;

/**
 * The FFI bridge for all events that can be thrown across the application.
 */
@Slf4j
public class EventPublisherBridge implements FxCallback<Event> {
    private final EventPublisher eventPublisher;
    private final FxChannel fxChannel;

    public EventPublisherBridge(EventPublisher eventPublisher, FxChannel fxChannel) {
        Objects.requireNonNull(eventPublisher, "eventPublisher cannot be null");
        Objects.requireNonNull(fxChannel, "fxChannel cannot be null");
        this.eventPublisher = eventPublisher;
        this.fxChannel = fxChannel;
        init();
    }

    private void init() {
        eventPublisher.register(PlayerStateEvent.class, event -> {
            var playerEvent = mapPlayerStateEvent(event);

            if (isBridgePublishAllowed(event))
                publishEvent(playerEvent);

            return event;
        }, EventPublisher.HIGHEST_ORDER);

        log.debug("Registering event bridge callback");

    }

    @Override
    public void callback(Event message) {
        switch (message.getType()) {
            case PLAYER_CHANGED -> {
                var event = message.getPlayerChanged();
                eventPublisher.publish(new PlayerChangedEvent(
                        this,
                        event.getOldPlayerId(),
                        event.getNewPlayerId(),
                        event.getNewPlayerName()
                ));
            }
            case PLAYER_STARTED -> eventPublisher.publish(new PlayerStartedEvent(this));
            case PLAYER_STOPPED -> eventPublisher.publish(new PlayerStoppedEvent(this));
            case PLAYBACK_STATE_CHANGED -> eventPublisher.publishEvent(new PlayerStateEvent(this, message.getPlaybackStateChanged().getNewState()));
            case WATCH_STATE_CHANGED -> {
            }
            case LOADING_STARTED -> {
            }
            case LOADING_COMPLETED -> {
            }
            case TORRENT_DETAILS_LOADED -> {
            }
            case CLOSE_PLAYER -> {
            }
            case UNRECOGNIZED -> {
            }
        }
    }

    private Event mapPlayerStateEvent(PlayerStateEvent playerStateEvent) {
        return Event.newBuilder()
                .setType(Event.EventType.PLAYBACK_STATE_CHANGED)
                .setPlaybackStateChanged(Event.PlaybackStateChanged.newBuilder()
                        .setNewState(playerStateEvent.getNewState())
                        .build())
                .build();
    }

    private void publishEvent(Event event) {
        fxChannel.send(event);
    }

    private boolean isBridgePublishAllowed(ApplicationEvent event) {
        return !(event.getSource() instanceof EventC);
    }
}
