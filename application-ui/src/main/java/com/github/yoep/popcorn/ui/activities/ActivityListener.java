package com.github.yoep.popcorn.ui.activities;

/**
 * Listener which listens on certain type of Activity.
 *
 * @param <T> The activity type to listen on.
 */
public interface ActivityListener<T extends Activity> {
    /**
     * Invoked when the activity is being registered in the {@link ActivityManager}.
     *
     * @param activity The activity that is being registered.
     */
    void onActive(T activity);
}
