package com.github.yoep.popcorn.backend.events;

import lombok.extern.slf4j.Slf4j;

import java.util.Objects;
import java.util.Queue;
import java.util.concurrent.ConcurrentLinkedQueue;
import java.util.function.Function;

/**
 * A simplistic event listener which allows the decoupling of components.
 * This event listener allows the ordering of the listeners within the chain as well as stopping the chain from continuing by consuming the value and
 * returning {@code null}.
 * <p>
 * <h2>Publish an event</h2>
 * <pre>
 * var publisher = new EventPublisher();
 * publisher.publish(myEvent);
 * </pre>
 * <p>
 * <h2>Consume event</h2>
 * <pre>
 * publisher.register(MyEvent.class, event -> {
 *     // do something with the event
 *     return null; // consume the event and stop the event chain
 * })
 * </pre>
 */
@Slf4j
public class EventPublisher {
    public static final int HIGHEST_ORDER = Integer.MIN_VALUE;
    public static final int LOWEST_ORDER = Integer.MAX_VALUE;

    private final Queue<ListenerHolder<? extends ApplicationEvent>> listeners = new ConcurrentLinkedQueue<>();
    private boolean useThreading = true;


    public EventPublisher() {
    }

    /**
     * Create a new event publisher with the given threading option.
     * When disabling the threading, the events will be invoked on the callers thread making it blocking.
     *
     * @param useThreading The indication if threading needs to be used.
     */
    public EventPublisher(boolean useThreading) {
        this.useThreading = useThreading;
    }

    /**
     * Notify all matching listeners registered with this event publisher.
     * Events may be any application events which extend the {@link ApplicationEvent} class.
     * <p>
     * The event handling is delegated to a new thread making it async.
     */
    public <T extends ApplicationEvent> void publish(T event) {
        if (event == null)
            return;

        if (useThreading) {
            new Thread(() -> doInternalPublish(event), "event-publisher").start();
        } else {
            doInternalPublish(event);
        }
    }

    /**
     * Notify all matching listeners registered with this event publisher.
     * Events may be any application events which extend the {@link ApplicationEvent} class.
     * <p>
     * The event handling is delegated to a new thread making it async.
     */
    public <T extends ApplicationEvent> void publishEvent(T event) {
        publish(event);
    }

    /**
     * Register a new event action which will be invoked when it's specified class is published.
     * It will also be trigger for all super types of the given class.
     *
     * @param clazz  The class to listen for.
     * @param action The action to invoke when the event is published.
     */
    public <T extends ApplicationEvent> void register(Class<T> clazz, Function<T, T> action) {
        register(clazz, action, 0);
    }

    /**
     * Register a new event action which will be invoked when it's specified class is published.
     * It will also be trigger for all super types of the given class.
     *
     * @param clazz  The class to listen for.
     * @param action The action to invoke when the event is published.
     * @param order  The order of this listener.
     */
    public <T extends ApplicationEvent> void register(Class<T> clazz, Function<T, T> action, int order) {
        Objects.requireNonNull(clazz, "clazz cannot be null");
        Objects.requireNonNull(action, "action cannot be null");
        listeners.add(new ListenerHolder<>(clazz, order, action));
    }

    private <T extends ApplicationEvent> void doInternalPublish(T event) {
        var eventType = event.getClass().getSimpleName();
        log.debug("Received event {}", event);

        try {
            // retrieve a list of listeners that need to be invoked
            var eventCopy = event;
            var eventChain = listeners.stream()
                    .filter(e -> e.clazz.isAssignableFrom(event.getClass()))
                    .map(e -> (ListenerHolder<T>) e)
                    .toList();

            log.trace("Invoking a total of {} listeners for {}", eventChain.size(), eventType);
            for (ListenerHolder<T> invocation : eventChain.stream().sorted().toList()) {
                eventCopy = invocation.action.apply(eventCopy);

                if (eventCopy == null) {
                    break;
                }
            }
        } catch (Exception ex) {
            log.error("An error occurred during the event loop, {}", ex.getMessage(), ex);
        }
    }

    private record ListenerHolder<T extends ApplicationEvent>(Class<T> clazz, int order, Function<T, T> action) implements Comparable<ListenerHolder<T>> {
        @Override
        public int compareTo(ListenerHolder o) {
            return Integer.compare(order, o.order);
        }
    }
}
