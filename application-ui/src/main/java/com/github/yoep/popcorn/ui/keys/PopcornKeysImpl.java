package com.github.yoep.popcorn.ui.keys;

import com.github.yoep.popcorn.ui.keys.bindings.popcorn_keys_media_key_pressed_t;
import com.github.yoep.popcorn.ui.keys.bindings.popcorn_keys_t;
import com.sun.jna.CallbackThreadInitializer;
import com.sun.jna.Native;
import com.sun.jna.StringArray;
import lombok.extern.slf4j.Slf4j;
import org.springframework.util.Assert;

import java.util.ArrayList;
import java.util.List;

@Slf4j
public class PopcornKeysImpl implements PopcornKeys {
    private final List<MediaKeyPressedListener> listeners = new ArrayList<>();
    private final MediaKeyPressedCallback mediaKeyPressedCallback = new MediaKeyPressedCallback();
    private final popcorn_keys_t instance;

    /**
     * Initialize a new {@link PopcornKeysImpl} instance.
     *
     * @param args The library arguments.
     */
    public PopcornKeysImpl(String... args) {
        log.trace("Initializing new popcorn keys instance");
        instance = PopcornKeysLib.popcorn_keys_new(args.length, new StringArray(args));

        if (instance == null) {
            throw new PopcornKeysException("Failed to initialize Popcorn Keys instance");
        }

        init();
    }

    //region Methods

    @Override
    public void addListener(MediaKeyPressedListener listener) {
        Assert.notNull(listener, "listener cannot be null");
        synchronized (listeners) {
            listeners.add(listener);
        }
    }

    @Override
    public void release() {
        log.debug("Releasing Popcorn Keys");
        // remove all listeners
        synchronized (listeners) {
            listeners.clear();
        }

        // release the library resources
        PopcornKeysLib.popcorn_keys_release(instance);
    }

    //endregion

    //region Functions

    private void init() {
        PopcornKeysLib.popcorn_keys_media_callback(instance, mediaKeyPressedCallback);
        log.debug("Popcorn keys has been initialized");
    }

    private void onMediaKeyPressed(int type) {
        log.trace("Received media key native type {}", type);

        try {
            var mediaKeyType = MediaKeyType.fromNativeValue(type);
            log.debug("Received media key event " + mediaKeyType);

            synchronized (listeners) {
                for (MediaKeyPressedListener listener : listeners) {
                    listener.onMediaKeyPressed(mediaKeyType);
                }
            }
        } catch (Exception ex) {
            log.error("An error occurred while processing the media key pressed event, " + ex.getMessage(), ex);
        }
    }

    //endregion

    private class MediaKeyPressedCallback implements popcorn_keys_media_key_pressed_t {
        private final CallbackThreadInitializer cti;

        MediaKeyPressedCallback() {
            this.cti = new CallbackThreadInitializer(true, false, "popcorn-keys-media-key-pressed");
            Native.setCallbackThreadInitializer(this, cti);
        }

        @Override
        public void callback(int type) {
            onMediaKeyPressed(type);
        }
    }
}
