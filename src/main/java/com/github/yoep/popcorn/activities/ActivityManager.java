package com.github.yoep.popcorn.activities;

/**
 * Manages the activities that occur within the JavaFX application.
 * This manager allows to easily manage activity events that occur within the JavaFX application and dispatch the events to the correct listeners.
 * {@link org.springframework.stereotype.Controller}'s can use this manager for loose coupling between each other.
 */
public interface ActivityManager {
    /**
     * Register the given activity listener.
     *
     * @param activity The activity class to listen on.
     * @param listener The listener to register.
     */
    <T extends Activity> void register(Class<T> activity, ActivityListener<T> listener);

    /**
     * Register the given activity to the manager.
     * This will invoke all listeners for this activity.
     *
     * @param activity The activity that is being activated.
     */
    void register(Activity activity);

    /**
     * Unregister the given listener from this manager.
     * This will stop the listener from receiving any activity events.
     *
     * @param listener The listener to unregister.
     * @param <T> The activity to which the listener is listening.
     */
    <T extends Activity> void unregister(ActivityListener<T> listener);
}
