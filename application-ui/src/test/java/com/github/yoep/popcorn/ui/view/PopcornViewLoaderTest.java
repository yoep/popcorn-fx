package com.github.yoep.popcorn.ui.view;

import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.IoC;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;

import java.util.concurrent.atomic.AtomicReference;
import java.util.function.Consumer;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.doAnswer;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class PopcornViewLoaderTest {
    @Mock
    private IoC instance;
    @Mock
    private ViewManager viewManager;
    @Mock
    private LocaleText localeText;
    @Mock
    private ApplicationConfig applicationConfig;

    @Test
    void testOnUiScaleChanged() {
        var newScale = 2.0f;
        var holder = new AtomicReference<Consumer<Float>>();
        doAnswer(invocation -> {
            holder.set(invocation.getArgument(0));
            return null;
        }).when(applicationConfig).setOnUiScaleChanged(isA(Consumer.class));
        var loader = new PopcornViewLoader(instance, applicationConfig, viewManager, localeText);

        var consumer = holder.get();
        consumer.accept(newScale);

        assertEquals(newScale, loader.getScale());
    }
}