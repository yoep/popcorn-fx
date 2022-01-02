package com.github.yoep.player.qt.player;

import com.github.yoep.player.qt.PopcornPlayerLib;
import com.github.yoep.player.qt.bindings.popcorn_player_duration_callback_t;
import com.github.yoep.player.qt.bindings.popcorn_player_state_callback_t;
import com.github.yoep.player.qt.bindings.popcorn_player_t;
import com.github.yoep.player.qt.bindings.popcorn_player_time_callback_t;
import com.sun.jna.CallbackThreadInitializer;
import com.sun.jna.Native;
import lombok.extern.slf4j.Slf4j;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

@Slf4j
class PopcornPlayerEventManager {
    private final List<PopcornPlayerEventListener> listeners = new ArrayList<>();
    private final StateCallback stateCallback = new StateCallback();
    private final TimeCallback timeCallback = new TimeCallback();
    private final DurationCallback durationCallback = new DurationCallback();

    PopcornPlayerEventManager(popcorn_player_t instance) {
        init(instance);
    }

    public void addListener(PopcornPlayerEventListener listener) {
        synchronized (listeners) {
            listeners.add(listener);
        }
    }

    public void removeListener(PopcornPlayerEventListener listener) {
        synchronized (listeners) {
            listeners.remove(listener);
        }
    }

    private void init(popcorn_player_t instance) {
        PopcornPlayerLib.popcorn_player_state_callback(instance, stateCallback);
        PopcornPlayerLib.popcorn_player_time_callback(instance, timeCallback);
        PopcornPlayerLib.popcorn_player_duration_callback(instance, durationCallback);
    }

    private void onStateChanged(int newState) {
        var state = PopcornPlayerState.of(newState);

        synchronized (listeners) {
            listeners.forEach(e -> e.onStateChanged(state));
        }
    }

    private void onTimeChanged(long newTime) {
        synchronized (listeners) {
            listeners.forEach(e -> e.onTimeChanged(newTime));
        }
    }

    private void onDurationChanged(long newDuration) {
        synchronized (listeners) {
            listeners.forEach(e -> e.onDurationChanged(newDuration));
        }
    }

    private class StateCallback implements popcorn_player_state_callback_t {
        private final CallbackThreadInitializer cti;

        StateCallback() {
            this.cti = new CallbackThreadInitializer(true, false, "popcorn-player-state");
            Native.setCallbackThreadInitializer(this, cti);
        }

        @Override
        public void callback(int newState) {
            onStateChanged(newState);
        }
    }

    private class TimeCallback implements popcorn_player_time_callback_t {
        private final CallbackThreadInitializer cti;

        TimeCallback() {
            this.cti = new CallbackThreadInitializer(true, false, "popcorn-player-time");
            Native.setCallbackThreadInitializer(this, cti);
        }

        @Override
        public void callback(String newValue) {
            Optional.ofNullable(newValue)
                    .map(Long::parseLong)
                    .ifPresentOrElse(PopcornPlayerEventManager.this::onTimeChanged, this::onCallbackValueEmpty);
        }

        private void onCallbackValueEmpty() {
            log.warn("Time callback value is NULL, unable to resolve callback correctly");
        }
    }

    private class DurationCallback implements popcorn_player_duration_callback_t {
        private final CallbackThreadInitializer cti;

        DurationCallback() {
            this.cti = new CallbackThreadInitializer(true, false, "popcorn-player-duration");
            Native.setCallbackThreadInitializer(this, cti);
        }

        @Override
        public void callback(String newValue) {
            Optional.ofNullable(newValue)
                    .map(Long::parseLong)
                    .ifPresentOrElse(PopcornPlayerEventManager.this::onDurationChanged, this::onCallbackValueEmpty);
        }

        private void onCallbackValueEmpty() {
            log.warn("Duration callback value is NULL, unable to resolve callback correctly");
        }
    }
}
