package com.github.yoep.popcorn.backend.utils;

import lombok.RequiredArgsConstructor;

import java.time.Duration;
import java.util.ArrayList;
import java.util.List;
import java.util.Timer;
import java.util.TimerTask;

@RequiredArgsConstructor
public class IdleTimer {
    private static final Timer TIMER = new Timer("IdleTimer", true);
    private final List<TimerTask> scheduledTasks = new ArrayList<>();
    /**
     * The idle timer duration before the {@link #onTimeout} action is called.
     */
    private final Duration duration;

    private Runnable onTimeout;

    //region Getters & Setters

    /**
     * Get the total number of scheduled tasks in this idle timer.
     * This total number also includes the current known cancelled tasks.
     *
     * @return Returns the total of schedule idle tasks.
     */
    public int getTotalScheduledTasks() {
        int total;

        synchronized (scheduledTasks) {
             total = scheduledTasks.size();
        }

        return total;
    }

    /**
     * Set the action which needs to be executed when the timeout is reached.
     *
     * @param onTimeout The action to execute.
     */
    public void setOnTimeout(Runnable onTimeout) {
        this.onTimeout = onTimeout;
    }

    //endregion

    //region Methods

    /**
     * Cancel any running idle timer and start a new one.
     * This will cancel any timer started by {@link #start()}.
     */
    public void runFromStart() {
        // cancel the previous schedule
        cancel();

        // schedule a new timeout task
        schedule();
    }

    /**
     * Start the idle timer.
     * This will schedule a new idle timer which won't cancel any already running idle timers.
     */
    public void start() {
        schedule();
    }

    /**
     * Stop the idle timer.
     * This will cancel any started timeout's which have not yet been finished.
     * If the timeout was finished and the {@link #onTimeout} action is executing, the action won't be interrupted.
     */
    public void stop() {
        cancel();
    }

    //endregion

    //region Functions

    private void cancel() {
        // cancel the individual TimerTask's
        // if the TIMER is cancelled, then it won't accept any new TimerTask's
        synchronized (scheduledTasks) {
            scheduledTasks.forEach(TimerTask::cancel);
        }

        // clean the references to the TimerTask's
        purge();
    }

    private void purge() {
        synchronized (scheduledTasks) {
            scheduledTasks.clear();
        }

        TIMER.purge();
    }

    private void schedule() {
        // if no timeout action is defined
        // ignore the schedule as it won't consequently do anything
        if (onTimeout == null)
            return;

        var task = new TimerTask() {
            @Override
            public void run() {
                onTimeout.run();
            }
        };

        synchronized (scheduledTasks) {
            scheduledTasks.add(task);
            TIMER.schedule(task, duration.toMillis());
        }
    }

    //endregion
}
