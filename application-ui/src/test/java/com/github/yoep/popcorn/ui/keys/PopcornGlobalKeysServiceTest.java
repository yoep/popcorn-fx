package com.github.yoep.popcorn.ui.keys;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.doAnswer;
import static org.mockito.Mockito.verify;

@ExtendWith(MockitoExtension.class)
class PopcornGlobalKeysServiceTest {
    @Mock
    private PopcornKeys popcornKeys;
    @InjectMocks
    private PopcornGlobalKeysService globalKeysService;

    @Test
    void testInit_whenInvoked_shouldRegisterListener() {
        var globalListener = new AtomicReference<MediaKeyPressedListener>();
        grabGlobalServiceListener(globalListener);

        globalKeysService.init();

        assertNotNull(globalListener.get());
    }

    @Test
    void testOnDestroy_whenInvoked_shouldReleaseThePopcornKeysResources() {
        globalKeysService.onDestroy();

        verify(popcornKeys).release();
    }

    @Test
    void testAddListener_whenPlayKeyIsPressed_shouldInvokedOnMediaPlay() {
        var invoked = new AtomicBoolean(false);
        var globalListener = new AtomicReference<MediaKeyPressedListener>();
        grabGlobalServiceListener(globalListener);
        var listener = new AbstractGlobalKeysListener() {
            @Override
            public void onMediaPlay() {
                invoked.set(true);
            }
        };

        initializeService(listener);
        globalListener.get().onMediaKeyPressed(MediaKeyType.PLAY);

        assertTrue(invoked.get());
    }

    @Test
    void testAddListener_whenPauseKeyIsPressed_shouldInvokedOnMediaPause() {
        var invoked = new AtomicBoolean(false);
        var globalListener = new AtomicReference<MediaKeyPressedListener>();
        grabGlobalServiceListener(globalListener);
        var listener = new AbstractGlobalKeysListener() {
            @Override
            public void onMediaPause() {
                invoked.set(true);
            }
        };

        initializeService(listener);
        globalListener.get().onMediaKeyPressed(MediaKeyType.PAUSE);

        assertTrue(invoked.get());
    }

    @Test
    void testAddListener_whenStopKeyIsPressed_shouldInvokedOnMediaStop() {
        var invoked = new AtomicBoolean(false);
        var globalListener = new AtomicReference<MediaKeyPressedListener>();
        grabGlobalServiceListener(globalListener);
        var listener = new AbstractGlobalKeysListener() {
            @Override
            public void onMediaStop() {
                invoked.set(true);
            }
        };

        initializeService(listener);
        globalListener.get().onMediaKeyPressed(MediaKeyType.STOP);

        assertTrue(invoked.get());
    }

    @Test
    void testAddListener_whenPreviousKeyIsPressed_shouldInvokedOnMediaPrevious() {
        var invoked = new AtomicBoolean(false);
        var globalListener = new AtomicReference<MediaKeyPressedListener>();
        grabGlobalServiceListener(globalListener);
        var listener = new AbstractGlobalKeysListener() {
            @Override
            public void onPreviousMedia() {
                invoked.set(true);
            }
        };

        initializeService(listener);
        globalListener.get().onMediaKeyPressed(MediaKeyType.PREVIOUS);

        assertTrue(invoked.get());
    }

    @Test
    void testAddListener_whenNextKeyIsPressed_shouldInvokedOnMediaNext() {
        var invoked = new AtomicBoolean(false);
        var globalListener = new AtomicReference<MediaKeyPressedListener>();
        grabGlobalServiceListener(globalListener);
        var listener = new AbstractGlobalKeysListener() {
            @Override
            public void onNextMedia() {
                invoked.set(true);
            }
        };

        initializeService(listener);
        globalListener.get().onMediaKeyPressed(MediaKeyType.NEXT);

        assertTrue(invoked.get());
    }

    private void grabGlobalServiceListener(AtomicReference<MediaKeyPressedListener> globalListener) {
        doAnswer(invocation -> {
            var listener = (MediaKeyPressedListener) invocation.getArgument(0);
            globalListener.set(listener);
            return null;
        }).when(popcornKeys).addListener(isA(MediaKeyPressedListener.class));
    }

    private void initializeService(AbstractGlobalKeysListener listener) {
        globalKeysService.init();
        globalKeysService.addListener(listener);
    }

    private abstract class AbstractGlobalKeysListener implements GlobalKeysListener {
        @Override
        public void onMediaPlay() {
            // no-op
        }

        @Override
        public void onMediaPause() {
            // no-op
        }

        @Override
        public void onMediaStop() {
            // no-op
        }

        @Override
        public void onPreviousMedia() {
            // no-op
        }

        @Override
        public void onNextMedia() {
            // no-op
        }
    }
}
