package com.github.yoep.popcorn.activities;

/**
 * Manages the activities that occur within the JavaFX application.
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
}
