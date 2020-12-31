package com.github.yoep.popcorn.ui.keys;

import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.util.Assert;

import javax.annotation.PostConstruct;
import javax.annotation.PreDestroy;
import javax.validation.constraints.NotNull;
import java.util.ArrayList;
import java.util.List;

@Slf4j
@RequiredArgsConstructor
public class PopcornGlobalKeysService implements GlobalKeysService {
    private final PopcornKeys popcornKeys;

    private final List<GlobalKeysListener> listeners = new ArrayList<>();

    //region Methods

    @Override
    public void addListener(@NotNull GlobalKeysListener listener) {
        Assert.notNull(listener, "listener cannot be null");
        synchronized (listeners) {
            listeners.add(listener);
        }
    }

    @Override
    public void removeListener(GlobalKeysListener listener) {
        synchronized (listeners) {
            listeners.remove(listener);
        }
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    void init() {
        popcornKeys.addListener(this::onMediaKeyPressed);
        log.info("Popcorn global keys service has been initialized");
    }

    //endregion

    //region PreDestroy

    @PreDestroy
    void onDestroy() {
        popcornKeys.release();
    }

    //endregion

    //region Functions

    private void onMediaKeyPressed(MediaKeyType type) {
        try {
            synchronized (listeners) {
                for (var listener : listeners) {
                    switch (type) {
                        case PLAY:
                            listener.onMediaPlay();
                            break;
                        case PAUSE:
                            listener.onMediaPause();
                            break;
                        case PREVIOUS:
                            listener.onPreviousMedia();
                            break;
                        case NEXT:
                            listener.onNextMedia();
                            break;
                        case STOP:
                            listener.onMediaStop();
                            break;
                        default:
                            // no-op
                            break;
                    }
                }
            }
        } catch (Exception ex) {
            // catch all exceptions as we don't want them to boil back up to the C library
            log.error("An unexpected error occurred while processing the media key press, " + ex.getMessage(), ex);
        }
    }

    //endregion
}
