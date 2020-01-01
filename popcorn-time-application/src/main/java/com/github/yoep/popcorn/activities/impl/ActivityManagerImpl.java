package com.github.yoep.popcorn.activities.impl;

import com.github.yoep.popcorn.activities.Activity;
import com.github.yoep.popcorn.activities.ActivityListener;
import com.github.yoep.popcorn.activities.ActivityManager;
import lombok.AllArgsConstructor;
import lombok.Getter;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.task.TaskExecutor;
import org.springframework.stereotype.Component;
import org.springframework.util.Assert;

import java.util.ArrayList;
import java.util.List;

@Slf4j
@Component
@RequiredArgsConstructor
public class ActivityManagerImpl implements ActivityManager {
    private final List<ListenerHolder> listeners = new ArrayList<>();
    private final TaskExecutor taskExecutor;

    @Override
    public <T extends Activity> void register(Class<T> activity, ActivityListener<T> listener) {
        Assert.notNull(activity, "activity cannot be null");
        Assert.notNull(listener, "listener cannot be null");
        synchronized (listeners) {
            listeners.add(new ListenerHolder<>(activity, listener));
        }
    }

    @Override
    public void register(Activity activity) {
        Assert.notNull(activity, "activity cannot be null");
        taskExecutor.execute(() -> {
            synchronized (listeners) {
                listeners.stream()
                        .filter(e -> e.getActivity().isAssignableFrom(activity.getClass()))
                        .map(ListenerHolder::getListener)
                        .forEach(e -> invokeActivityListener(activity, e));
            }
        });
    }

    @Override
    public <T extends Activity> void unregister(ActivityListener<T> listener) {
        Assert.notNull(listener, "listener cannot be null");
        listeners.removeIf(e -> e.getListener() == listener);
    }

    private static void invokeActivityListener(Activity activity, ActivityListener<Activity> listener) {
        // make sure we are exception safe invoking each listener,
        // as they might throw an error which prevents other listeners from receiving any event
        try {
            listener.onActive(activity);
        } catch (Exception ex) {
            log.error(ex.getMessage(), ex);
        }
    }

    /**
     * Holds information about an activity listener.
     * This holder makes it easier for lookup of the correct listeners instead of using a complex reflection setup.
     *
     * @param <T> The activity class this listener listens on.
     */
    @Getter
    @AllArgsConstructor
    private static class ListenerHolder<T extends Activity> {
        private final Class<T> activity;
        private final ActivityListener<T> listener;
    }
}
