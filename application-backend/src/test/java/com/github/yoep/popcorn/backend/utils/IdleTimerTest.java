package com.github.yoep.popcorn.backend.utils;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.parallel.Execution;
import org.junit.jupiter.api.parallel.ExecutionMode;

import java.time.Duration;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicInteger;

import static org.junit.jupiter.api.Assertions.*;

@Execution(ExecutionMode.CONCURRENT)
class IdleTimerTest {
    @Test
    void testStart_whenInvoked_shouldInvokeOnTimeoutActionWhenDurationHasExpired() throws InterruptedException {
        var result = new AtomicBoolean();
        var timer = new IdleTimer(Duration.ofSeconds(1));

        timer.setOnTimeout(() -> result.set(true));
        timer.start();

        Thread.sleep(1200);

        assertTrue(result.get());
    }

    @Test
    void testStart_whenOnTimeoutHasNotBeenSet_shouldNotScheduleTimerTask() {
        var timer = new IdleTimer(Duration.ofSeconds(1));

        timer.start();

        assertEquals(0, timer.getTotalScheduledTasks());
    }

    @Test
    void testStop_whenInvoked_shouldCancelAnyScheduledTimer() throws InterruptedException {
        var result = new AtomicBoolean();
        var timer = new IdleTimer(Duration.ofSeconds(1));

        timer.setOnTimeout(() -> result.set(true));
        timer.start();
        timer.stop();

        Thread.sleep(1100);

        assertFalse(result.get());
    }

    @Test
    void testRunFromStart_whenInvoked_shouldCancelAnyPreviousScheduledTasks() throws InterruptedException {
        var result = new AtomicInteger();
        var timer = new IdleTimer(Duration.ofSeconds(1));

        timer.setOnTimeout(result::incrementAndGet);
        timer.start();
        timer.runFromStart();

        Thread.sleep(1300);

        assertEquals(1, result.get());
    }
}
