package com.github.yoep.popcorn.ui.keys;

import com.github.yoep.popcorn.ui.keys.conditions.ConditionalOnPopcornKeys;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;
import org.springframework.util.Assert;

import javax.annotation.PostConstruct;
import javax.annotation.PreDestroy;
import javax.validation.constraints.NotNull;
import java.util.ArrayList;
import java.util.List;

@Slf4j
@Service
@ConditionalOnPopcornKeys
public class PopcornGlobalKeysService implements GlobalKeysService {
    private final List<GlobalKeysListener> listeners = new ArrayList<>();
    private PopcornKeys popcornKeys;

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
    private void init() {
        try {
            var level = getLogLevel();
            var args = new String[]{"PopcornKeys", "-l", level};

            popcornKeys = new PopcornKeys(args);
        } catch (UnsatisfiedLinkError ex) {
            log.error("Failed to load the popcorn keys library, " + ex.getMessage(), ex);
        }
    }

    //endregion

    //region PreDestroy

    @PreDestroy
    private void onDestroy() {
        if (popcornKeys != null) {
            popcornKeys.release();
            popcornKeys = null;
        }
    }

    //endregion

    //region Functions

    private String getLogLevel() {
        if (log.isTraceEnabled()) {
            return "trace";
        } else if (log.isDebugEnabled()) {
            return "debug";
        }

        return "info";
    }

    //endregion
}
